use crate::error::Result;

pub type EnvVar = (String, String);
pub type SplitEnvResult = (Vec<String>, Vec<EnvVar>);

pub fn parse_env_kv(input: &str) -> std::result::Result<EnvVar, String> {
    let Some((key, value)) = input.split_once('=') else {
        return Err("Expected KEY=VALUE".to_string());
    };

    let key = key.trim();
    if key.is_empty() {
        return Err("Environment variable key cannot be empty".to_string());
    }

    Ok((key.to_string(), value.to_string()))
}

pub fn split_env_flags(args: &[String]) -> Result<SplitEnvResult> {
    let mut remaining = Vec::new();
    let mut envs = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let token = &args[i];

        if token == "--env" {
            let Some(pair) = args.get(i + 1) else {
                return Err("--env requires a KEY=VALUE argument".into());
            };
            let parsed = parse_env_kv(pair).map_err(|e| e.to_string())?;
            envs.push(parsed);
            i += 2;
            continue;
        }

        if let Some(pair) = token.strip_prefix("--env=") {
            let parsed = parse_env_kv(pair).map_err(|e| e.to_string())?;
            envs.push(parsed);
            i += 1;
            continue;
        }

        remaining.push(token.clone());
        i += 1;
    }

    Ok((remaining, envs))
}

#[cfg(test)]
mod tests {
    use super::{parse_env_kv, split_env_flags};

    #[test]
    fn parse_env_kv_works() {
        let (k, v) = parse_env_kv("A=B").expect("parse env");
        assert_eq!(k, "A");
        assert_eq!(v, "B");
    }

    #[test]
    fn split_env_flags_extracts_repeatable_values() {
        let args = vec![
            "msg".to_string(),
            "--env".to_string(),
            "A=1".to_string(),
            "--env=B=2".to_string(),
            "tail".to_string(),
        ];

        let (remaining, envs) = split_env_flags(&args).expect("split env flags");
        assert_eq!(remaining, vec!["msg", "tail"]);
        assert_eq!(
            envs,
            vec![
                ("A".to_string(), "1".to_string()),
                ("B".to_string(), "2".to_string())
            ]
        );
    }
}
