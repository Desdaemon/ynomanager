use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Game {
  pub name: String,
  pub yno_translation: Option<String>,
  pub patches: Vec<Patch>,
}

#[derive(Deserialize, Debug)]
pub struct Patch {
  version: Semver,
  prerelease: Option<Prerelease>,
  #[serde(default = "default_true")]
  pub standalone: bool,
  pub version_name: String,
  pub link: String,
  pub path: String,
}

fn default_true() -> bool {
  true
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct FullSemverVersion {
  semver: Semver,
  prerelease: Option<Prerelease>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum SemverUpgrade {
  Prerelease,
  Patch,
  Minor,
  Major,
}

impl FullSemverVersion {
  fn parse(data: &str) -> FullSemverVersion {
    let data = data.strip_prefix('v').unwrap_or(data);
    let mut parts = data.split('-');
    let semver = parts.next().unwrap();
    let prerelease = parts.next().map(|pr| {
      let mut parts = pr.split('.');
      let kind = parts.next().unwrap();
      let revision = parts.next().unwrap().parse().unwrap();
      Prerelease {
        kind: kind.to_string(),
        revision,
      }
    });
    let mut parts = semver.split('.');
    let major = parts.next().unwrap().parse().unwrap();
    let minor = parts.next().unwrap().parse().unwrap();
    let patch = parts.next().unwrap().parse().unwrap();
    FullSemverVersion {
      semver: Semver {
        major,
        minor,
        patch,
      },
      prerelease,
    }
  }
  fn upgrade_kind(&self, other: &Self) -> SemverUpgrade {
    if self.semver.major < other.semver.major {
      SemverUpgrade::Major
    } else if self.semver.minor < other.semver.minor {
      SemverUpgrade::Minor
    } else if self.semver.patch < other.semver.patch {
      SemverUpgrade::Patch
    } else if self.prerelease.is_some() && other.prerelease.is_none() {
      SemverUpgrade::Prerelease
    } else if self.prerelease.is_some() && other.prerelease.is_some() {
      let self_pr = self.prerelease.as_ref().unwrap();
      let other_pr = other.prerelease.as_ref().unwrap();
      if self_pr.kind != other_pr.kind {
        todo!("handle different prerelease kinds")
      } else if self_pr.revision < other_pr.revision {
        SemverUpgrade::Prerelease
      } else {
        SemverUpgrade::Patch
      }
    } else {
      SemverUpgrade::Patch
    }
  }
}

impl Patch {
  fn full_semver_version(&self) -> FullSemverVersion {
    FullSemverVersion {
      semver: self.version,
      prerelease: self.prerelease.clone(),
    }
  }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semver {
  #[serde(rename = "0")]
  pub major: u32,
  #[serde(rename = "1")]
  pub minor: u32,
  #[serde(rename = "2")]
  pub patch: u32,
}

#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Prerelease {
  #[serde(rename = "0")]
  pub kind: String,
  #[serde(rename = "1")]
  pub revision: u32,
}

/// `patches` must be sorted in increasing semver order.
pub fn calculate_patch<'a>(patches: &'a [Patch], current: Option<&str>) -> Vec<&'a Patch> {
  let current = current.unwrap_or("0.0.0");
  let semver = FullSemverVersion::parse(current);
  let mut out = vec![];

  let Some(latest_patch) = patches.last() else {
    return out;
  };

  let target_upgrade = semver.upgrade_kind(&latest_patch.full_semver_version());
  let Some((idx, _)) = patches.iter().enumerate().find(|(_, p)| {
    let patch_ver = p.full_semver_version();
    patch_ver > semver && semver.upgrade_kind(&patch_ver) >= target_upgrade
  }) else {
    return out;
  };

  if let Some((idx, _)) = patches
    .iter()
    .enumerate()
    .skip(idx)
    .rfind(|(_, p)| p.standalone)
  {
    out.extend(&patches[idx..]);
  } else {
    out.extend(&patches[idx..]);
  }

  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_only_take_last_standalone_and_subsequent() {
    let patches = vec![
      Patch {
        version: Semver {
          major: 1,
          minor: 0,
          patch: 0,
        },
        prerelease: None,
        standalone: false,
        version_name: "1.0.0".to_string(),
        link: "https://example.com/1.0.0".to_string(),
        path: "1.0.0".to_string(),
      },
      Patch {
        version: Semver {
          major: 1,
          minor: 0,
          patch: 1,
        },
        prerelease: None,
        standalone: true,
        version_name: "1.0.1".to_string(),
        link: "https://example.com/1.0.1".to_string(),
        path: "1.0.1".to_string(),
      },
      Patch {
        version: Semver {
          major: 1,
          minor: 0,
          patch: 2,
        },
        prerelease: None,
        standalone: false,
        version_name: "1.0.2".to_string(),
        link: "https://example.com/1.0.2".to_string(),
        path: "1.0.2".to_string(),
      },
    ];
    let result = calculate_patch(&patches, None);
    assert!(matches!(
      &result[..],
      [
        Patch {
          version: Semver {
            major: 1,
            minor: 0,
            patch: 1,
          },
          ..
        },
        Patch {
          version: Semver {
            major: 1,
            minor: 0,
            patch: 2,
          },
          ..
        }
      ]
    ));
  }
}
