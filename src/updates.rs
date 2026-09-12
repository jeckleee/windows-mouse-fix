use serde::Deserialize;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    draft: bool,
    prerelease: bool,
}

pub fn check(repository: &str, beta: bool) -> Result<Option<(String, String)>, String> {
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|s| {
            s.is_empty()
                || !s
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        })
    {
        return Err("请填写此 Windows 项目的 GitHub 发布仓库：所有者/仓库名".into());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent(concat!("WindowsMouseFix/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let releases: Vec<Release> = client
        .get(format!(
            "https://api.github.com/repos/{repository}/releases?per_page=100"
        ))
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json())
        .map_err(|e| e.to_string())?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let latest = releases
        .into_iter()
        .filter(|r| !r.draft && (beta || !r.prerelease))
        .filter_map(|r| {
            semver::Version::parse(r.tag_name.trim_start_matches('v'))
                .ok()
                .map(|v| (v, r))
        })
        .filter(|(v, _)| beta || v.pre.is_empty())
        .max_by(|(a, _), (b, _)| a.cmp(b));
    Ok(latest
        .filter(|(v, _)| *v > current)
        .map(|(_, r)| (r.tag_name, r.html_url)))
}
