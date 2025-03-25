mod test_utils {
    // Macro for declarative test setup
    macro_rules! test_suite {
        ($name:ident { $($test:ident $body:block)* }) => {
            mod $name {
                use super::*;
                $(
                    #[test]
                    #[cfg_attr(feature = "rt-feedback", allow(unused))] // Handle feature-specific tests
                    fn $test() -> Result<(), String> {
                        setup_test_env!();
                        $body
                        Ok(())
                    }
                )*
            }
        };
    }

    // Macro for batch file additions
    macro_rules! add_files {
        ($files:expr => { $($path:literal $content:expr),* $(,)? }) => {
            $(
                $files.add_file($path.to_string(), $content.to_vec());
            )*
        };
    }

    // Snapshots with context-aware naming
    macro_rules! assert_position_snapshot {
        ($pos:expr) => {{
            use insta::assert_snapshot;
            let formatted = format!(
                "Start: {}:{}\nEnd: {}:{}",
                $pos.start_line(),
                $pos.start_column(),
                $pos.end_line(),
                $pos.end_column()
            );
            assert_snapshot!(
                concat!(module_path!(), "::", function!(), "::", line!()),
                formatted
            );
        }};
    }

    // Test environment setup with automatic cleanup
    macro_rules! setup_test_env {
        () => {
            let _guard = {
                #[cfg(feature = "rt-feedback")]
                let feedback = crate::RuntimeFeedback::default();
            };
        };
    }
    // Core test macro with Insta snapshot configuration
    macro_rules! exhaustive_test_suite {
        ($name:ident { $($test:ident $body:tt)* }) => {
            mod $name {
                use super::*;
                use crate::*;

                $(
                    #[test]
                    fn $test() -> Result<(), String> {
                        let _settings = insta::Settings::new()
                            .set_snapshot_suffix(format!(
                                "{}__{}__{}",
                                module_path!(),
                                stringify!($test),
                                line!()
                            ));

                        let result: Result<_, String> = (|| -> Result<_, String> {
                            $body
                        })();

                        match result {
                            Ok(val) => {
                                // Automatic snapshot type detection
                                match &val {
                                    _ if std::any::type_name::<()>() == std::any::type_name_of_val(&val) => {}
                                    _ => insta::assert_yaml_snapshot!(val, {
                                        ".path_to_id"=> insta::sorted_redaction(),
                                    }),
                                }
                                Ok(())
                            }
                            Err(e) => {
                                insta::assert_snapshot!("ERROR", &e);
                                Err(e)
                            }
                        }
                    }
                )*
            }
        };
    }

    // Multi-feature test combinator
    macro_rules! feature_combination_test {
        ($($feature:literal),*, $name:ident, $body:block) => {
            #[cfg(all($(feature = $feature),*))]
            mod $name {
                use super::*;
                #[test]
                fn test() -> Result<(), String> {
                    setup_test_env!();
                    $body
                }
            }
        };
       }

    // Edge case generator

    pub(crate) use {
        add_files, exhaustive_test_suite, feature_combination_test, setup_test_env, test_suite,
    };
}

#[cfg(not(feature = "rt-feedback"))]
mod basic {
    use super::*;
    use test_utils::*;

    test_suite!(core_functionality {
        test_basic_file_operations {
            let mut files = SourceFilesMap::<u8>::new();
            add_files!(files => {
                "src/main.rs" b"fn main() {}",
                "src/lib.rs" b"pub mod utils;"
            });
            files.finalize()?;

            let file_id = files.get_id("src/main.rs").unwrap();
            let pos = create_absolute_position(file_id, 1, 1, 1, 10);
            assert_position_snapshot!(pos);
        }

        test_multi_line_position {
            let mut files = SourceFilesMap::<u8>::new();
            add_files!(files => {
                "data.txt" b"Line1\nLine2\nLine3"
            });
            files.finalize()?;

            let file_id = files.get_id("data.txt").unwrap();
            let pos = create_relative_position(2, 1, 3, 5);
            assert_position_snapshot!(pos);
        }
    });
}

// Example usage to show the simple integration
test_utils::exhaustive_test_suite!(core_functionality {
    test_basic_operations {
        let mut files = SourceFilesMap::<u8>::new();
        test_utils::add_files!(files => {
            "main.rs" b"fn main() {}",
            "lib.rs" b"pub mod utils;"
        });
        files.finalize()?;
        Ok(files)
    }

    test_position_encoding {
        let pos = create_absolute_position(1_u8, 10, 5, 15, 20);
        Ok(pos)
    }

    test_max_files {
        let mut files = SourceFilesMap::<u8>::new();
        for i in 0..u8::MAX {
            files.add_file(format!("file_{}.rs", i), vec![]);
        }
        files.finalize()
    }

    test_duplicate_paths {
        let mut files = SourceFilesMap::<u8>::new();
        test_utils::add_files!(files => {
            "dup.rs" b"content",
            "dup.rs" b"different"
        });
        files.finalize()
    }
});

#[cfg(feature = "rt-feedback")]
#[cfg(test)]
mod rt_feedback {
    use crate::*;
    use std::sync::{Arc, Mutex};

    // Macro to simplify file addition with optional content
    macro_rules! add_file {
        ($files:expr, $path:expr) => {
            $files.add_file($path.to_string(), Vec::new())
        };
        ($files:expr, $path:expr, $content:expr) => {
            $files.add_file($path.to_string(), $content.to_vec())
        };
    }

    // Helper function to create a runtime feedback context
    fn create_feedback_context() -> Arc<Mutex<RuntimeFeedback>> {
        Arc::new(Mutex::new(RuntimeFeedback::default()))
    }

    #[test]
    fn feedback_file_tracking() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create SourceFilesMap with feedback context
        let mut files_map = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));

        // Add some files using the new macro
        add_file!(files_map, "src/main.rs");
        add_file!(files_map, "src/lib.rs");
        add_file!(files_map, "tests/integration.rs");

        // Finalize the map to trigger feedback tracking
        files_map.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 3);
        assert_eq!(feedback_data.usage_count, 1);

        Ok(())
    }

    #[test]
    fn feedback_multiple_finalizations() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create multiple file maps with the same feedback context
        let mut files_map1 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map1, "project1/src/main.rs");
        files_map1.finalize()?;

        let mut files_map2 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map2, "project2/src/lib.rs");
        add_file!(files_map2, "project2/src/utils.rs");
        files_map2.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 2); // Second finalization overwrites first
        assert_eq!(feedback_data.usage_count, 2);

        Ok(())
    }

    #[test]
    fn feedback_file_size_tracking() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create SourceFilesMap with feedback context
        let mut files_map = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));

        // Simulate files with different sizes
        add_file!(files_map, "small.rs");
        add_file!(files_map, "medium.rs");
        add_file!(files_map, "large.rs");

        // Finalize the map
        files_map.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 3);

        Ok(())
    }

    #[test]
    fn feedback_context_sharing() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create multiple file maps sharing the same feedback context
        let mut files_map1 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map1, "project1/main.rs");
        files_map1.finalize()?;

        let mut files_map2 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map2, "project2/lib.rs");
        files_map2.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 1); // Only most recent finalization counts
        assert_eq!(feedback_data.usage_count, 2);

        Ok(())
    }
}
#[cfg(feature = "view")]
#[cfg(test)]
mod view {
    use super::*;
    use crate::*;
    use test_utils::*;
    test_suite!(view {
        test_multi_line_view {
            let mut files = SourceFilesMap::<u8>::new();
            add_files!(files => {
                "multiline.txt" b"First\nSecond\nThird"
            });
            files.finalize()?;

            let file_id = files.get_id("multiline.txt").unwrap();
            let pos = create_relative_position(1, 1, 3, 5);
            let content = unsafe { std::str::from_utf8_unchecked(files.view(file_id, &pos).unwrap()) };
            insta::assert_debug_snapshot!(content);
        }
    });
}
#[cfg(test)]
mod comprehensive {
    use super::*;
    use crate::*;
    use test_utils::*;

    // Utilizing the exhaustive_test_suite macro for comprehensive testing
    exhaustive_test_suite!(source_files_map {
        test_basic_file_operations {
            let mut files = SourceFilesMap::<u16>::new();
            add_files!(files => {
                "src/main.rs" b"fn main() { println!(\"Hello, world!\"); }",
                "src/lib.rs" b"pub mod utils;\npub fn helper() -> bool { true }",
                "README.md" b"# Project Documentation\n\nThis is a sample project."
            });
            files.finalize()?;

            // Verify file ids can be retrieved
            assert!(files.get_id("src/main.rs").is_some());
            assert!(files.get_id("src/lib.rs").is_some());
            assert!(files.get_id("README.md").is_some());
            Ok(files)
        }

        test_file_id_uniqueness {
            let mut files = SourceFilesMap::<u16>::new();
            add_files!(files => {
                "project1/src/main.rs" b"fn main() {}",
                "project2/src/main.rs" b"fn main() {}"
            });
            files.finalize()?;

            let id1 = files.get_id("project1/src/main.rs").unwrap();
            let id2 = files.get_id("project2/src/main.rs").unwrap();

            assert_ne!(id1, id2, "File IDs should be unique across different paths");
            Ok((id1, id2))
        }

        test_position_creation {
            let mut files = SourceFilesMap::<u16>::new();
            add_files!(files => {
                "test.rs" b"fn example() {\n    let x = 42;\n    println!(\"Value: {}\", x);\n}"
            });
            files.finalize()?;

            let file_id = files.get_id("test.rs").unwrap();

            // Test absolute position creation
            let abs_pos = create_absolute_position(file_id, 2, 1, 3, 20);

            // Test relative position creation
            let rel_pos = create_relative_position(1, 1, 3, 20);

            Ok((abs_pos, rel_pos))
        }
    });

    // Feature combination test for conditional compilation
    test_utils::feature_combination_test!("rt-feedback", "view", file_tracking_with_view, {
        let mut files = SourceFilesMap::<u16>::new();
        add_files!(files => {
            "tracked_view_file.rs" b"// Tracked file with view support"
        });
        files.finalize()?;

        let file_id = files.get_id("tracked_view_file.rs").unwrap();
        let pos = create_relative_position(1, 1, 1, 10);
        let view_result = files.view(file_id, &pos);

        assert!(view_result.is_some());
        Ok(())
    });
}
