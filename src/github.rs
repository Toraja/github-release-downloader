use std::io::Read;

use regex::Regex;
use serde::Deserialize;
use url::Url;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Deserialize)]
pub struct Release {
    pub assets: Vec<Asset>,
}

pub fn to_api_url(url: &Url) -> Result<Url, AppError> {
    if url.host_str() != Some("github.com") {
        return Err(AppError::InvalidHost(
            url.host_str().unwrap_or("<none>").to_string(),
        ));
    }

    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.filter(|seg| !seg.is_empty()).collect())
        .unwrap_or_default();

    if segments.len() < 2 {
        return Err(AppError::InvalidPath(url.path().to_string()));
    }

    let owner = segments[0];
    let repo = segments[1];

    Url::parse(&format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    ))
    .map_err(|e| AppError::UrlConstruct(e.to_string()))
}

pub fn fetch_release(api_url: &Url) -> Result<Release, AppError> {
    let mut req = ureq::get(api_url.as_str())
        .header("User-Agent", "ghrls")
        .header("Accept", "application/vnd.github+json");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    let mut response = req
        .call()
        .map_err(|e| AppError::ApiRequest(e.to_string()))?;

    response
        .body_mut()
        .read_json::<Release>()
        .map_err(|e| AppError::JsonParse(e.to_string()))
}

pub fn select_asset<'a>(assets: &'a [Asset], pattern: &Regex) -> Result<&'a Asset, AppError> {
    let matches: Vec<&Asset> = assets
        .iter()
        .filter(|a| pattern.is_match(&a.name))
        .collect();

    match matches.len() {
        0 => {
            let available: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
            Err(AppError::NoMatch {
                pattern: pattern.to_string(),
                available: available.join("\n  "),
            })
        }
        1 => Ok(matches[0]),
        _ => {
            let matched: Vec<&str> = matches.iter().map(|a| a.name.as_str()).collect();
            Err(AppError::MultipleMatches {
                pattern: pattern.to_string(),
                matched: matched.join("\n  "),
            })
        }
    }
}

pub fn fetch_asset(asset: &Asset) -> Result<impl Read + 'static, AppError> {
    let response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", "ghrls")
        .call()
        .map_err(|e| AppError::Download(e.to_string()))?;

    Ok(response.into_body().into_reader())
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use regex::Regex;
    use url::Url;

    use super::*;
    use crate::error::AppError;

    fn make_asset(name: &str) -> Asset {
        Asset {
            name: name.to_string(),
            browser_download_url: format!("https://example.com/{}", name),
        }
    }

    #[test]
    fn test_no_matching_assets() {
        let assets = vec![
            make_asset("gh_2.40.1_linux_amd64.tar.gz"),
            make_asset("gh_2.40.1_darwin_amd64.tar.gz"),
        ];
        let pattern = Regex::new("windows").unwrap();
        let err = select_asset(&assets, &pattern);
        assert_matches!(
            err,
            Err(AppError::NoMatch{pattern:p, available:a})
                if p == "windows" && a.contains("gh_2.40.1_linux_amd64.tar.gz") && a.contains("gh_2.40.1_darwin_amd64.tar.gz")
        );
    }

    #[test]
    fn test_multiple_matching_assets() {
        let assets = vec![
            make_asset("gh_2.40.1_linux_amd64.tar.gz"),
            make_asset("gh_2.40.1_linux_arm64.tar.gz"),
            make_asset("gh_2.40.1_darwin_amd64.tar.gz"),
        ];
        let pattern = Regex::new("linux").unwrap();
        let err = select_asset(&assets, &pattern);
        assert_matches!(
            err,
            Err(AppError::MultipleMatches{pattern:p, matched:m})
                if p == "linux" && m.contains("gh_2.40.1_linux_amd64.tar.gz") && m.contains("gh_2.40.1_linux_arm64.tar.gz") && !m.contains("darwin")
        );
    }

    #[test]
    fn test_single_matching_asset() {
        let assets = vec![
            make_asset("gh_2.40.1_linux_amd64.tar.gz"),
            make_asset("gh_2.40.1_darwin_amd64.tar.gz"),
        ];
        let pattern = Regex::new(r"linux_amd64").unwrap();
        let asset = select_asset(&assets, &pattern).unwrap();
        assert_eq!(asset.name, "gh_2.40.1_linux_amd64.tar.gz");
    }

    #[test]
    fn test_standard_url() {
        let url = Url::parse("https://github.com/owner/repo").unwrap();
        assert!(to_api_url(&url).is_ok());
        assert_eq!(
            to_api_url(&url).unwrap().as_str(),
            "https://api.github.com/repos/owner/repo/releases/latest"
        );
    }

    #[test]
    fn test_trailing_slash_url() {
        let url = Url::parse("https://github.com/owner/repo/").unwrap();
        assert!(to_api_url(&url).is_ok());
        assert_eq!(
            to_api_url(&url).unwrap().as_str(),
            "https://api.github.com/repos/owner/repo/releases/latest"
        );
    }

    #[test]
    fn test_non_github_domain() {
        let url = Url::parse("https://gitlab.com/owner/repo").unwrap();
        assert_matches!(to_api_url(&url), Err(AppError::InvalidHost(h)) if h == "gitlab.com");
    }

    #[test]
    fn test_missing_repo_segment() {
        let url = Url::parse("https://github.com/owner").unwrap();
        assert_matches!(to_api_url(&url), Err(AppError::InvalidPath(p)) if p == "/owner");
    }
}
