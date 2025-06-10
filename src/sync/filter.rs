use std::str::FromStr;

use version_compare::Cmp;

#[derive(Debug, PartialEq, Eq)]
pub enum VersionFilter {
    Any,
    Op(String, version_compare::Cmp),
}

#[derive(Debug, PartialEq)]
pub struct SyncFilter {
    pub name: String,
    pub version: Option<VersionFilter>,
}

impl FromStr for SyncFilter {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err("Empty input");
        }

        let operators = ["!=", "=", ">=", ">", "<=", "<"];
        let mut found_index = None;
        let mut found_op = None;

        'outer: for i in 0..input.len() {
            for op in &operators {
                if input[i..].starts_with(op) {
                    if i == 0 {
                        return Err("Missing identifier");
                    }
                    found_index = Some(i);
                    found_op = Some(*op);
                    break 'outer;
                }
            }
        }

        if let (Some(i), Some(op)) = (found_index, found_op) {
            let name = input[..i].to_string();
            let version_str = input[i + op.len()..].to_string();
            if version_str.is_empty() {
                return Err("Version string is empty");
            }
            if op == "=" && version_str == "*" {
                return Ok(SyncFilter {
                    name,
                    version: Some(VersionFilter::Any),
                });
            }
            let cmp = match op {
                "=" => Cmp::Eq,
                "!=" => Cmp::Ne,
                ">" => Cmp::Gt,
                ">=" => Cmp::Ge,
                "<" => Cmp::Lt,
                "<=" => Cmp::Le,
                _ => unreachable!(),
            };
            Ok(SyncFilter {
                name,
                version: Some(VersionFilter::Op(version_str, cmp)),
            })
        } else {
            Ok(SyncFilter {
                name: input.to_string(),
                version: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_version() {
        assert_eq!(
            SyncFilter::from_str("ident"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: None,
            })
        );
    }

    #[test]
    fn test_any_version() {
        assert_eq!(
            SyncFilter::from_str("ident=*"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Any),
            })
        );
    }

    #[test]
    fn test_gt_version() {
        assert_eq!(
            SyncFilter::from_str("ident>1.0.0"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Op("1.0.0".to_string(), Cmp::Gt)),
            })
        );
    }

    #[test]
    fn test_eq_version() {
        assert_eq!(
            SyncFilter::from_str("ident=1.0.0"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Op("1.0.0".to_string(), Cmp::Eq)),
            })
        );
    }

    #[test]
    fn test_ge_version() {
        assert_eq!(
            SyncFilter::from_str("ident>=1.0.0"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Op("1.0.0".to_string(), Cmp::Ge)),
            })
        );
    }

    #[test]
    fn test_lt_version() {
        assert_eq!(
            SyncFilter::from_str("ident<1.0.0"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Op("1.0.0".to_string(), Cmp::Lt)),
            })
        );
    }

    #[test]
    fn test_le_version() {
        assert_eq!(
            SyncFilter::from_str("ident<=1.0.0"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Op("1.0.0".to_string(), Cmp::Le)),
            })
        );
    }

    #[test]
    fn test_ne_version() {
        assert_eq!(
            SyncFilter::from_str("ident!=1.0.0"),
            Ok(SyncFilter {
                name: "ident".to_string(),
                version: Some(VersionFilter::Op("1.0.0".to_string(), Cmp::Ne)),
            })
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(SyncFilter::from_str(""), Err("Empty input"));
    }

    #[test]
    fn test_missing_identifier() {
        assert_eq!(SyncFilter::from_str("=*"), Err("Missing identifier"));
    }

    #[test]
    fn test_missing_version() {
        assert_eq!(
            SyncFilter::from_str("ident>"),
            Err("Version string is empty")
        );
    }
}
