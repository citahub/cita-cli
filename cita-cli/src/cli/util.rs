use std::str::FromStr;

use clap::{App, ArgMatches};

use cita_tool::{remove_0x, Address, H256, PrivateKey, U256};

use interactive::GlobalConfig;

/// Get url from arg match
pub fn get_url<'a>(m: &'a ArgMatches, config: &'a GlobalConfig) -> &'a str {
    match m.subcommand() {
        (_, Some(m)) => {
            if m.subcommand().1.is_some() {
                get_url(m, config)
            } else {
                m.value_of("url")
                    .unwrap_or_else(|| config.get_url().as_str())
            }
        }
        _ => config.get_url().as_str(),
    }
}

/// the hexadecimal or numeric type string resolves to u64
pub fn parse_u64(height: &str) -> Result<u64, String> {
    match is_hex(height) {
        Ok(()) => Ok(u64::from_str_radix(remove_0x(height), 16).map_err(|err| format!("{}", err))?),
        _ => match height.parse::<u64>() {
            Ok(number) => Ok(number),
            Err(e) => Err(format!("{:?}", e)),
        },
    }
}

/// Attempt to resolve the private key
pub fn parse_privkey(hash: &str) -> Result<PrivateKey, String> {
    is_hex(hash)?;
    Ok(PrivateKey::from_str(remove_0x(hash))?)
}

pub fn is_hex(hex: &str) -> Result<(), String> {
    let tmp = hex.as_bytes();
    if tmp.len() < 2 {
        Err("Must be a hexadecimal string".to_string())
    } else if tmp[..2] == b"0x"[..] || tmp[..2] == b"0X"[..] {
        Ok(())
    } else {
        Err("Must hex string".to_string())
    }
}

pub fn parse_height(height: &str) -> Result<(), String> {
    match height {
        "latest" | "earliest" => Ok(()),
        _ => match parse_u64(height) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{:?}", e)),
        },
    }
}

pub fn parse_u256(value: &str) -> Result<U256, String> {
    match is_hex(value) {
        Ok(_) => Ok(U256::from_str(remove_0x(value))
            .map_err(|_| String::from("Value can't parse into u256"))?),
        Err(_) => {
            Ok(U256::from_dec_str(value).map_err(|_| String::from("Value can't parse into u256"))?)
        }
    }
}

pub fn h256_validator(value: &str) -> Result<(), String> {
    is_hex(value)?;
    H256::from_str(remove_0x(value))
        .map(|_| ())
        .map_err(|_| "Invalid H256".to_string())
}

pub fn parse_address(value: &str) -> Result<(), String> {
    is_hex(value)?;
    Address::from_str(remove_0x(value))
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(feature = "blake2b_hash")]
pub fn blake2b(m: &ArgMatches, config: &GlobalConfig) -> bool {
    m.is_present("blake2b") || config.blake2b()
}
#[cfg(not(feature = "blake2b_hash"))]
pub fn blake2b(_m: &ArgMatches, _config: &GlobalConfig) -> bool {
    false
}

/// Search command tree
pub fn search_app<'a, 'b>(
    app: &App<'a, 'b>,
    prefix: &Option<Vec<String>>,
    commands: &mut Vec<Vec<String>>,
) {
    for inner_app in &app.p.subcommands {
        if inner_app.p.subcommands.is_empty() {
            if prefix.is_some() {
                let mut sub_command = prefix.clone().unwrap();
                sub_command.push(inner_app.p.meta.name.clone());
                commands.push(sub_command);
            } else {
                commands.push(vec![inner_app.p.meta.name.clone()]);
            }
        } else {
            let prefix: Option<Vec<String>> = if prefix.is_some() {
                prefix.clone().map(|mut x| {
                    x.push(inner_app.p.meta.name.clone());
                    x
                })
            } else {
                Some(vec![inner_app.p.meta.name.clone()])
            };

            search_app(inner_app, &prefix, commands);
        };
    }
}
