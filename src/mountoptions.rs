/// Settings provided via the command-line flag `-o`.
#[derive(Debug)]
pub struct MountOptions {
    /// The initial random number.
    pub seed: u64,
}

impl Default for MountOptions {
    fn default() -> MountOptions {
        MountOptions {
            // TODO: replace PID by a proper source of entropy.
            seed: std::process::id() as u64,
        }
    }
}

/// Errors that may come up when updating `MountOptions` from a string of parameters.
#[derive(PartialEq, Eq, Debug)]
pub enum UpdateError {
    /// Parameter expects a hexadecimal number, but the value contains non-hexadecimal digits.
    NonHexValue {
        parameter: String,
        value: String,
    },

    /// Parameter requires a value, but none/empty one was provided.
    NoValue {
        parameter: String,
    },

    ValueTooLong {
        parameter: String,
        value: String,
        max_allowed_length: usize,
    },

    UnsupportedParameter {
        parameter: String,
        value: String,
    },
}

impl MountOptions {
    /// Given a string like "seed=123ff,files_count=100500", store the values in the appropriate
    /// fields of `MountOptions`.
    pub fn update_from(&mut self, parameters: &str) -> Result<(), UpdateError> {
        for key_value in parameters.split(',') {
            let (parameter, value) = split_key_value(key_value);
            match parameter {
                "seed" => {
                    if value.is_empty() {
                        return Err(UpdateError::NoValue {
                            parameter: parameter.to_string(),
                        });
                    }

                    if let Some(_) = value.find(|c: char| !c.is_ascii_hexdigit()) {
                        return Err(UpdateError::NonHexValue {
                            parameter: parameter.to_string(),
                            value: value.to_string(),
                        });
                    }

                    if value.len() > 16 {
                        return Err(UpdateError::ValueTooLong {
                            max_allowed_length: 16,
                            parameter: parameter.to_string(),
                            value: value.to_string(),
                        });
                    }

                    self.seed = u64::from_str_radix(value, 16).expect("Failed to parse string as hexadecimal number despite running checks on it beforehand");
                }

                _ => {
                    return Err(UpdateError::UnsupportedParameter {
                        parameter: parameter.to_string(),
                        value: value.to_string(),
                    })
                }
            }
        }

        Ok(())
    }
}

/// Split a key-value pair into the key and the value (which are separated by `=`).
///
/// If `kv` contains no separator, the whole input is treated as key, and the value is empty. If
/// `kv` contains multiple equals signs, the first one is treated as separator. If `kv` is empty,
/// both the key and the value are empty.
fn split_key_value(kv: &str) -> (&str, &str) {
    if let Some(offset) = kv.find('=') {
        let (key, mut value) = kv.split_at(offset);
        // Drop the leading `=`.
        value = &value[1..];

        (key, value)
    } else {
        let key = kv;

        (key, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_seed() {
        let mut sut = MountOptions::default();

        sut.update_from("seed=123").unwrap();
        assert_eq!(sut.seed, 0x123);

        sut.update_from("seed=f00").unwrap();
        assert_eq!(sut.seed, 0xf00);

        sut.update_from("seed=ba7ba,seed=f256555895d306f0").unwrap();
        assert_eq!(sut.seed, 0xf256555895d306f0);
    }

    #[test]
    fn returns_err_non_hex_value() {
        let mut sut = MountOptions::default();

        assert_eq!(
            sut.update_from("seed=hello"),
            Err(UpdateError::NonHexValue {
                parameter: "seed".to_string(),
                value: "hello".to_string()
            })
        );
    }

    #[test]
    fn returns_err_value_too_long() {
        let mut sut = MountOptions::default();

        assert_eq!(
            sut.update_from("seed=0123456789abcdef0"),
            Err(UpdateError::ValueTooLong {
                max_allowed_length: 16,
                parameter: "seed".to_string(),
                value: "0123456789abcdef0".to_string(),
            })
        );
    }

    #[test]
    fn returns_err_unsupported_parameter() {
        let mut sut = MountOptions::default();

        assert_eq!(
            sut.update_from("knob=11turns"),
            Err(UpdateError::UnsupportedParameter {
                parameter: "knob".to_string(),
                value: "11turns".to_string(),
            })
        );

        assert_eq!(
            sut.update_from("will this work?"),
            Err(UpdateError::UnsupportedParameter {
                parameter: "will this work?".to_string(),
                value: String::new()
            })
        );
    }

    #[test]
    fn t_split_key_value() {
        assert_eq!(split_key_value(""), ("", ""));
        assert_eq!(split_key_value("hello"), ("hello", ""));
        assert_eq!(split_key_value("key="), ("key", ""));
        assert_eq!(split_key_value("another=example"), ("another", "example"));
        assert_eq!(
            split_key_value("welcome=to=PlentyFS"),
            ("welcome", "to=PlentyFS")
        );
    }

    use proptest::prelude::*;

    proptest! {
        // We want to make sure that under "normal conditions", `update_from()` updates the seed
        // and returns `OK(())`.
        //
        // "Normal conditions" are easy to generate: a valid seed is a non-mepty hexadecimal string
        // no longer than 16 characters. We prepend "seed=" to it, and get a valid mount option.
        //
        // That would be enough to check that `update_from()` doesn't panic, but there is a chance
        // that the seed we generate is the same as the default one that `MountOptions` generated
        // upon construction. In that case, we won't be able to tell if the seed was updated or
        // not.
        //
        // To get around that, we generate *two* seeds, and ensure they are different. That way, if
        // `update_from()` doesn't update the seed, we'll notice because the intermediate state
        // will be the same as the final state. And we will still notice if `update_from()` panics.
        #[test]
        fn works_for_valid_seeds(seed1 in "[0-9a-f]{1,16}", seed2 in "[0-9a-f]{1,16}") {
            // We can't simply do `seed1 != seed2`, because e.g. "9" and "09" are different
            // *strings* but are actually the same *value*. So, before comparing them, we strip any
            // leading zeroes.
            let prefixless_seed1 = seed1.trim_start_matches('0');
            let prefixless_seed2 = seed2.trim_start_matches('0');
            prop_assume!(prefixless_seed1 != prefixless_seed2);

            let mut sut = MountOptions::default();
            sut.update_from(&format!("seed={}", seed1)).unwrap();

            let intermediate_result = sut.seed;

            sut.update_from(&format!("seed={}", seed2)).unwrap();

            assert_ne!(intermediate_result, sut.seed);
        }

        #[test]
        fn update_from_never_panics(parameter in ".*") {
            let mut sut = MountOptions::default();
            // We don't care if it returns an `Err()` since this test is only concerned with
            // panics.
            let _ = sut.update_from(&parameter);
        }
    }
}
