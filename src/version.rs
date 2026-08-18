use anyhow::*;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Version {
    // major.minor.patch(-pre)?(+build)?
    Semantic {
        major: usize,
        minor: usize,
        patch: usize,
        pre: Option<String>,
        build: Option<String>,
    },
    // epoch.major.minor.patch(-pre)?(+build)?
    Epoch {
        epoch: usize,
        major: usize,
        minor: usize,
        patch: usize,
        pre: Option<String>,
        build: Option<String>,
    },
    // r{epoch}.{major}.{minor_patch}(-pre)?(+build)?
    Romantic {
        epoch: usize,
        major: usize,
        minor_patch: usize,

        pre: Option<String>,
        build: Option<String>,
    },
}

impl Default for Version {
    fn default() -> Self {
        return Version::Semantic {
            major: 0,
            minor: 1,
            patch: 0,
            pre: None,
            build: None,
        };
    }
}

impl Version {
    pub fn compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Version::Semantic { major: a_major, .. },
                Version::Semantic { major: b_major, .. },
            ) => a_major == b_major,

            (
                Version::Epoch {
                    epoch: a_epoch,
                    major: a_major,
                    ..
                },
                Version::Epoch {
                    epoch: b_epoch,
                    major: b_major,
                    ..
                },
            ) => a_epoch == b_epoch && a_major == b_major,

            (
                Version::Romantic {
                    epoch: a_epoch,
                    major: a_major,
                    ..
                },
                Version::Romantic {
                    epoch: b_epoch,
                    major: b_major,
                    ..
                },
            ) => a_epoch == b_epoch && a_major == b_major,

            _ => false,
        }
    }

    pub fn to_version_string(&self) -> String {
        let (base, pre, build) = match self {
            Version::Semantic {
                major,
                minor,
                patch,
                pre,
                build,
            } => (
                format!("{major}.{minor}.{patch}"),
                pre.as_deref(),
                build.as_deref(),
            ),
            Version::Epoch {
                epoch,
                major,
                minor,
                patch,
                pre,
                build,
            } => (
                format!("{epoch}.{major}.{minor}.{patch}"),
                pre.as_deref(),
                build.as_deref(),
            ),
            Version::Romantic {
                epoch,
                major,
                minor_patch,
                pre,
                build,
            } => (
                format!("r{epoch}.{major}.{minor_patch}"),
                pre.as_deref(),
                build.as_deref(),
            ),
        };

        let with_pre = match pre {
            Some(pre) if !pre.is_empty() => format!("{base}-{pre}"),
            _ => base,
        };

        match build {
            Some(build) if !build.is_empty() => format!("{with_pre}+{build}"),
            _ => with_pre,
        }
    }

    fn from_version_string(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        ensure!(!trimmed.is_empty(), "version string cannot be empty");

        let (pre_build, build) = match trimmed.split_once('+') {
            Some((left, right)) => {
                if right.is_empty() {
                    return Err(anyhow!("build metadata cannot be empty"));
                }
                (left, Some(right.to_string()))
            }
            None => (trimmed, None),
        };

        let (core, pre) = match pre_build.split_once('-') {
            Some((left, right)) => {
                if right.is_empty() {
                    return Err(anyhow!("pre-release cannot be empty"));
                }
                (left, Some(right.to_string()))
            }
            None => (pre_build, None),
        };

        if let Some(stripped) = core.strip_prefix('r') {
            let parts: Vec<&str> = stripped.split('.').collect();
            if parts.len() != 3 {
                return Err(anyhow!("invalid romantic version: {input}"));
            }

            let epoch = parts[0]
                .parse::<usize>()
                .map_err(|_| anyhow!("invalid epoch in version: {input}"))?;
            let major = parts[1]
                .parse::<usize>()
                .map_err(|_| anyhow!("invalid major in version: {input}"))?;
            let minor_patch = parts[2]
                .parse::<usize>()
                .map_err(|_| anyhow!("invalid minor_patch in version: {input}"))?;

            return Ok(Version::Romantic {
                epoch,
                major,
                minor_patch,
                pre,
                build,
            });
        }

        let parts: Vec<&str> = core.split('.').collect();
        match parts.len() {
            3 => {
                let major = parts[0]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid major in version: {input}"))?;
                let minor = parts[1]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid minor in version: {input}"))?;
                let patch = parts[2]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid patch in version: {input}"))?;
                Ok(Version::Semantic {
                    major,
                    minor,
                    patch,
                    pre,
                    build,
                })
            }
            4 => {
                let epoch = parts[0]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid epoch in version: {input}"))?;
                let major = parts[1]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid major in version: {input}"))?;
                let minor = parts[2]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid minor in version: {input}"))?;
                let patch = parts[3]
                    .parse::<usize>()
                    .map_err(|_| anyhow!("invalid patch in version: {input}"))?;
                Ok(Version::Epoch {
                    epoch,
                    major,
                    minor,
                    patch,
                    pre,
                    build,
                })
            }
            _ => Err(anyhow!("invalid version format: {input}")),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_version_string())
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        Version::from_version_string(input)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_version_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_version_string(&s).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn version_deserializes_from_semantic_string() {
        let version: Version = serde_json::from_str("\"0.1.0\"").expect("valid semantic version");
        match version {
            Version::Semantic {
                major,
                minor,
                patch,
                ..
            } => {
                assert_eq!(major, 0);
                assert_eq!(minor, 1);
                assert_eq!(patch, 0);
            }
            _ => panic!("expected semantic version"),
        }
    }

    #[test]
    fn version_serializes_to_string() {
        let version = Version::Epoch {
            epoch: 1,
            major: 2,
            minor: 3,
            patch: 4,
            pre: Some("beta.1".to_string()),
            build: Some("exp.sha".to_string()),
        };
        let json = serde_json::to_string(&version).expect("serializes");
        assert_eq!(json, "\"1.2.3.4-beta.1+exp.sha\"");
    }

    #[test]
    fn romantic_version_uses_r_prefix() {
        let version: Version = serde_json::from_str("\"r0.1.0\"").expect("valid romantic version");
        match version {
            Version::Romantic {
                epoch,
                major,
                minor_patch,
                ..
            } => {
                assert_eq!(epoch, 0);
                assert_eq!(major, 1);
                assert_eq!(minor_patch, 0);
            }
            _ => panic!("expected romantic version"),
        }

        let json = serde_json::to_string(&Version::Romantic {
            epoch: 0,
            major: 1,
            minor_patch: 0,
            pre: None,
            build: None,
        })
        .expect("serializes romantic version");
        assert_eq!(json, "\"r0.1.0\"");
    }
}
