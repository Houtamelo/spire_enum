use super::*;

// Test enum with cfg attributes on variants
#[delegated_enum]
pub enum FeatureGatedEnum {
    AlwaysAvailable(i32),

    #[cfg(feature = "cond_comp")]
    TestFeature(String),

    #[cfg(target_os = "linux")]
    LinuxOnly(Vec<u8>),

    #[cfg(target_os = "windows")]
    WindowsOnly {
        handle: u64,
    },

    #[cfg(debug_assertions)]
    DebugOnly,

    #[cfg(not(debug_assertions))]
    ReleaseOnly(f64),

    #[cfg(all(unix, target_pointer_width = "64"))]
    Unix64Bit(usize),

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    X86Architecture(bool),
}

// Test enum with more complex cfg conditions
#[delegated_enum]
pub enum ComplexCfgEnum<T> {
    Base(T),

    #[cfg(all(feature = "cond_comp_advanced", not(target_os = "windows")))]
    AdvancedNonWindows(T, String),

    #[cfg(any(feature = "cond_comp_beta", feature = "cond_comp_experimental"))]
    BetaOrExperimental {
        value: T,
        metadata: Vec<String>,
    },

    #[cfg(target_family = "unix")]
    UnixFamily,
}

// Test delegated enum with cfg on variants for table generation
#[variant_type_table]
#[variant_generic_table]
pub enum ConfigurableSettings {
    BaseSettings(String),

    #[cfg(feature = "cond_comp_graphics")]
    GraphicsSettings(GraphicsConfig),

    #[cfg(feature = "cond_comp_audio")]
    AudioSettings(AudioConfig),

    #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
    NetworkSettings(NetworkConfig),
}

#[discriminant_generic_table]
pub enum ConfigurableSettingsWithDiscriminants {
    BaseSettings,
    #[cfg(feature = "cond_comp_graphics")]
    GraphicsSettings,
    #[cfg(feature = "cond_comp_audio")]
    AudioSettings,
    #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
    NetworkSettings,
}

// Support structs for the table test
#[cfg(feature = "cond_comp_graphics")]
#[derive(PartialEq, Debug)]
pub struct GraphicsConfig {
    pub resolution: (u32, u32),
    pub vsync: bool,
}

#[cfg(feature = "cond_comp_audio")]
#[derive(PartialEq, Debug)]
pub struct AudioConfig {
    pub volume:   f32,
    pub channels: u8,
}

#[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
#[derive(PartialEq, Debug)]
pub struct NetworkConfig {
    pub timeout: std::time::Duration,
    pub max_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_available_variant() {
        let value = FeatureGatedEnum::AlwaysAvailable(42);
        match value {
            FeatureGatedEnum::AlwaysAvailable(x) => assert_eq!(x, 42),
            _ => panic!("Expected AlwaysAvailable variant"),
        }
    }

    #[cfg(feature = "cond_comp")]
    #[test]
    fn test_feature_gated_variant() {
        let value = FeatureGatedEnum::TestFeature("test".to_string());
        match value {
            FeatureGatedEnum::TestFeature(s) => assert_eq!(s, "test"),
            _ => panic!("Expected TestFeature variant"),
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_only_variant() {
        let value = FeatureGatedEnum::LinuxOnly(vec![1, 2, 3]);
        match value {
            FeatureGatedEnum::LinuxOnly(data) => assert_eq!(data, vec![1, 2, 3]),
            _ => panic!("Expected LinuxOnly variant"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_only_variant() {
        let value = FeatureGatedEnum::WindowsOnly { handle: 12345 };
        match value {
            FeatureGatedEnum::WindowsOnly { handle } => assert_eq!(handle, 12345),
            _ => panic!("Expected WindowsOnly variant"),
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_only_variant() {
        let value = FeatureGatedEnum::DebugOnly;
        assert!(matches!(value, FeatureGatedEnum::DebugOnly));
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_release_only_variant() {
        let value = FeatureGatedEnum::ReleaseOnly(3.14);
        match value {
            FeatureGatedEnum::ReleaseOnly(x) => assert_eq!(x, 3.14),
            _ => panic!("Expected ReleaseOnly variant"),
        }
    }

    #[cfg(all(unix, target_pointer_width = "64"))]
    #[test]
    fn test_unix_64bit_variant() {
        let value = FeatureGatedEnum::Unix64Bit(1024);
        match value {
            FeatureGatedEnum::Unix64Bit(size) => assert_eq!(size, 1024),
            _ => panic!("Expected Unix64Bit variant"),
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_x86_architecture_variant() {
        let value = FeatureGatedEnum::X86Architecture(true);
        match value {
            FeatureGatedEnum::X86Architecture(flag) => assert!(flag),
            _ => panic!("Expected X86Architecture variant"),
        }
    }

    #[test]
    fn test_complex_cfg_base_variant() {
        let value = ComplexCfgEnum::Base(42);
        match value {
            ComplexCfgEnum::Base(x) => assert_eq!(x, 42),
            _ => panic!("Expected Base variant"),
        }
    }

    #[cfg(all(feature = "cond_comp_advanced", not(target_os = "windows")))]
    #[test]
    fn test_advanced_non_windows_variant() {
        let value = ComplexCfgEnum::AdvancedNonWindows(100, "cond_comp_advanced".to_string());
        match value {
            ComplexCfgEnum::AdvancedNonWindows(num, text) => {
                assert_eq!(num, 100);
                assert_eq!(text, "cond_comp_advanced");
            }
            _ => panic!("Expected AdvancedNonWindows variant"),
        }
    }

    #[cfg(any(feature = "cond_comp_beta", feature = "cond_comp_experimental"))]
    #[test]
    fn test_beta_or_experimental_variant() {
        let value = ComplexCfgEnum::BetaOrExperimental {
            value: "test",
            metadata: vec!["meta1".to_string(), "meta2".to_string()],
        };
        match value {
            ComplexCfgEnum::BetaOrExperimental { value, metadata } => {
                assert_eq!(value, "test");
                assert_eq!(metadata.len(), 2);
            }
            _ => panic!("Expected BetaOrExperimental variant"),
        }
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_unix_family_variant() {
        let value = ComplexCfgEnum::<i32>::UnixFamily;
        assert!(matches!(value, ComplexCfgEnum::UnixFamily));
    }

    #[test]
    fn test_configurable_settings_base() {
        let settings = ConfigurableSettings::BaseSettings("base config".to_string());
        match settings {
            ConfigurableSettings::BaseSettings(config) => {
                assert_eq!(config, "base config");
            }
            #[allow(unreachable_patterns)]
            _ => panic!("Expected BaseSettings variant"),
        }
    }

    #[cfg(feature = "cond_comp_graphics")]
    #[test]
    fn test_graphics_settings_variant() {
        let graphics = GraphicsConfig {
            resolution: (1920, 1080),
            vsync: true,
        };
        let settings = ConfigurableSettings::GraphicsSettings(graphics);
        match settings {
            ConfigurableSettings::GraphicsSettings(config) => {
                assert_eq!(config.resolution, (1920, 1080));
                assert!(config.vsync);
            }
            _ => panic!("Expected GraphicsSettings variant"),
        }
    }

    #[cfg(feature = "cond_comp_audio")]
    #[test]
    fn test_audio_settings_variant() {
        let audio = AudioConfig {
            volume:   0.8,
            channels: 2,
        };
        let settings = ConfigurableSettings::AudioSettings(audio);
        match settings {
            ConfigurableSettings::AudioSettings(config) => {
                assert_eq!(config.volume, 0.8);
                assert_eq!(config.channels, 2);
            }
            _ => panic!("Expected AudioSettings variant"),
        }
    }

    #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
    #[test]
    fn test_network_settings_variant() {
        let network = NetworkConfig {
            timeout: std::time::Duration::from_secs(30),
            max_connections: 100,
        };
        let settings = ConfigurableSettings::NetworkSettings(network);
        match settings {
            ConfigurableSettings::NetworkSettings(config) => {
                assert_eq!(config.timeout, std::time::Duration::from_secs(30));
                assert_eq!(config.max_connections, 100);
            }
            _ => panic!("Expected NetworkSettings variant"),
        }
    }

    // Test that conditional compilation works with pattern matching
    #[test]
    fn test_pattern_matching_with_cfg() {
        let value = FeatureGatedEnum::AlwaysAvailable(999);

        let result = match value {
            FeatureGatedEnum::AlwaysAvailable(x) => format!("always: {}", x),

            #[cfg(feature = "cond_comp")]
            FeatureGatedEnum::TestFeature(s) => format!("test: {}", s),

            #[cfg(target_os = "linux")]
            FeatureGatedEnum::LinuxOnly(_) => "linux".to_string(),

            #[cfg(debug_assertions)]
            FeatureGatedEnum::DebugOnly => "debug".to_string(),

            _ => "other".to_string(),
        };

        assert_eq!(result, "always: 999");
    }

    // Test compilation with different cfg combinations
    #[test]
    fn test_cfg_compilation_variants_exist() {
        // This test just ensures that the enum compiles correctly
        // with the current cfg settings and that available variants work

        // AlwaysAvailable should always be available
        let _always = FeatureGatedEnum::AlwaysAvailable(1);

        // Test that we can create instances of conditionally compiled variants
        // when their conditions are met
        #[cfg(debug_assertions)]
        {
            let _debug = FeatureGatedEnum::DebugOnly;
        }

        #[cfg(not(debug_assertions))]
        {
            let _release = FeatureGatedEnum::ReleaseOnly(1.0);
        }

        // Verify that the enum type itself is usable
        fn _accept_enum(_e: FeatureGatedEnum) {}
        _accept_enum(FeatureGatedEnum::AlwaysAvailable(42));
    }
}

// Integration tests for table generation with cfg attributes
#[cfg(test)]
mod table_tests {
    use super::*;

    #[test]
    fn test_configurable_settings_table_creation() {
        // Test that the generated table type exists and can be instantiated
        // Note: This test will only compile if the variant_type_table macro
        // properly handles cfg attributes

        #[allow(unused)]
        let var_ty_table = ConfigurableSettingsVariantTypeTable::new(
            "base".to_string(),
            #[cfg(feature = "cond_comp_graphics")]
            GraphicsConfig {
                resolution: (800, 600),
                vsync: false,
            },
            #[cfg(feature = "cond_comp_audio")]
            AudioConfig {
                volume:   1.0,
                channels: 2,
            },
            #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
            NetworkConfig {
                timeout: std::time::Duration::from_secs(10),
                max_connections: 50,
            },
        );

        #[allow(unused)]
        let var_gen_table = ConfigurableSettingsVariantGenericTable::new(
            "base",
            #[cfg(feature = "cond_comp_graphics")]
            "vulkan",
            #[cfg(feature = "cond_comp_audio")]
            "fmod",
            #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
            "tcp",
        );

        #[allow(unused)]
        let discrim_table = ConfigurableSettingsWithDiscriminantsDiscriminantTable::new(
            50,
            #[cfg(feature = "cond_comp_graphics")]
            100,
            #[cfg(feature = "cond_comp_audio")]
            150,
            #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
            200,
        );

        #[cfg(feature = "cond_comp_graphics")]
        {
            assert_eq! {
                *var_ty_table.get::<GraphicsConfig>(),
                GraphicsConfig {
                    resolution: (800, 600),
                    vsync: false,
                }
            }

            assert_eq! {
                *var_gen_table.get::<GraphicsConfig>(),
                "vulkan"
            }

            assert_eq! {
                *discrim_table.get(ConfigurableSettingsWithDiscriminants::GraphicsSettings),
                100
            }
        }

        #[cfg(feature = "cond_comp_audio")]
        {
            assert_eq! {
                *var_ty_table.get::<AudioConfig>(),
                AudioConfig {
                    volume: 1.0,
                    channels: 2,
                }
            }

            assert_eq! {
                *var_gen_table.get::<AudioConfig>(),
                "fmod"
            }

            assert_eq! {
                *discrim_table.get(ConfigurableSettingsWithDiscriminants::AudioSettings),
                150
            }
        }

        #[cfg(all(feature = "cond_comp_networking", not(target_arch = "wasm32")))]
        {
            assert_eq! {
                *var_ty_table.get::<NetworkConfig>(),
                NetworkConfig {
                    timeout: std::time::Duration::from_secs(10),
                    max_connections: 50,
                }
            }

            assert_eq! {
                *var_gen_table.get::<NetworkConfig>(),
                "tcp"
            }

            assert_eq! {
                *discrim_table.get(ConfigurableSettingsWithDiscriminants::NetworkSettings),
                200
            }
        }
    }
}
