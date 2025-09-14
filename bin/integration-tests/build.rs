//! Integration Test Generation Build Script
//!
//! This build script automatically discovers integration test functions and generates:
//! 1. Individual `#[tokio::test]` wrappers in `OUT_DIR/integration_tests.rs`
//! 2. Programmatic access via `Vec<TestCase>` in `OUT_DIR/test_registry.rs`
//!
//! The generated files are included via `include!()` macro to keep them out of the source tree.
//! Test functions are discovered by looking for functions named `test_*`.

use std::collections::HashSet;
use std::path::Path;
use std::{env, fs};

const TEST_PREFIX: &str = "test_";

/// Main entry point for the build script.
///
/// This function:
/// 1. Scans all Rust files in `src/` directory recursively
/// 2. Discovers test functions named `test_*`
/// 3. Generates integration test wrappers in `OUT_DIR/integration_tests.rs`
/// 4. Generates test case vector in `OUT_DIR/test_registry.rs`
///
/// The build script will re-run when:
/// - Any file in `src/` changes
/// - The `build.rs` file itself changes
fn main() {
    println!("cargo:rerun-if-changed=src/tests/");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:info=Running build script to generate integration tests");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    let test_cases = collect_test_cases();
    println!("cargo:info=Found {} test cases", test_cases.len());

    // Generate tokio test wrappers in OUT_DIR
    let integration_path = out_path.join("integration_tests.rs");
    let integration_code = generate_integration_tests(&test_cases);
    fs::write(&integration_path, integration_code).unwrap();
    println!("cargo:info=Generated tokio test wrappers in {}", integration_path.display());

    // Generate Vec<TestCase> in OUT_DIR
    let generated_path = out_path.join("test_registry.rs");
    let generated_code = generate_test_case_vector(&test_cases);
    fs::write(&generated_path, generated_code).unwrap();
    println!("cargo:info=Generated test case vector in {}", generated_path.display());
}

/// Information about a discovered integration test function.
///
/// This struct holds metadata for each test function found during the build process.
#[derive(Debug)]
struct TestCaseInfo {
    /// Display name for the test case (typically same as function_name)
    name: String,
    /// Test category derived from the file name (e.g., "client", "swap_transaction")
    category: String,
    /// The actual function name that implements the test
    function_name: String,
}

/// Discovers all integration test functions across all source files.
///
/// This function recursively scans the `src/tests` directory for Rust files and extracts
/// test case information from functions named `test_*`.
///
/// # Returns
///
/// A vector of [`TestCaseInfo`] structs containing metadata for each discovered test function.
///
/// # Example
///
/// ```
/// let test_cases = collect_test_cases();
/// println!("Found {} test cases", test_cases.len());
/// ```
fn collect_test_cases() -> Vec<TestCaseInfo> {
    let mut test_cases = Vec::new();
    let src_dir = Path::new("src/tests");

    if src_dir.exists() && src_dir.is_dir() {
        collect_test_cases_recursive(src_dir, &mut test_cases);
    }

    test_cases
}

/// Recursively scans directories for test functions.
fn collect_test_cases_recursive(current_dir: &Path, test_cases: &mut Vec<TestCaseInfo>) {
    for entry in fs::read_dir(current_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            collect_test_cases_recursive(&path, test_cases);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let mut file_test_cases = collect_test_cases_from_file(&path);
            test_cases.append(&mut file_test_cases);
        }
    }
}

/// Extracts test case information from a single Rust source file.
///
/// This function parses a Rust file's content using simple text processing
/// and identifies functions that start with `test_`.
///
/// # Arguments
///
/// * `file_path` - Path to the Rust source file to analyze
///
/// # Returns
///
/// A vector of [`TestCaseInfo`] structs for all test functions found in the file.
/// Returns an empty vector if the file cannot be read or parsed.
fn collect_test_cases_from_file(file_path: &Path) -> Vec<TestCaseInfo> {
    let mut test_cases = Vec::new();

    // Extract category from file path (e.g., "src/tests/client.rs" -> "client")
    let category = match extract_category_from_path(file_path) {
        Some(cat) => cat,
        None => return test_cases, // Skip files that don't match the expected pattern
    };

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return test_cases,
    };

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for function definitions that start with test_
        if let Some(function_name) = parse_test_function_name(trimmed) {
            test_cases.push(TestCaseInfo {
                name: function_name.clone(),
                category: category.clone(),
                function_name,
            });
        }
    }

    test_cases
}

/// Extracts the test category name from a file path.
///
/// The category is derived from the filename (without extension) and is used to
/// organize tests logically. For example, `src/tests/client.rs` produces the
/// category `"client"`.
///
/// # Arguments
///
/// * `file_path` - Path to the test file
///
/// # Returns
///
/// * `Some(String)` - The category name if the path is valid and points to a test file
/// * `None` - If the path doesn't match the expected pattern or points to `mod.rs`
///
/// # Examples
///
/// ```
/// let path = Path::new("src/tests/client.rs");
/// assert_eq!(extract_category_from_path(&path), Some("client".to_string()));
///
/// let mod_path = Path::new("src/tests/mod.rs");
/// assert_eq!(extract_category_from_path(&mod_path), None);
/// ```
fn extract_category_from_path(file_path: &Path) -> Option<String> {
    // Extract the filename without extension from paths like "src/tests/client.rs"
    let file_stem = file_path.file_stem()?.to_str()?;

    // Skip mod.rs and other special files
    if file_stem == "mod" {
        return None;
    }

    // Verify this is in the tests directory
    if !file_path.to_str()?.contains("src/tests/") {
        return None;
    }

    Some(file_stem.to_string())
}

/// Determines if a function should be treated as an integration test.
///
/// Looks for public function definitions that start with [`TEST_PREFIX`].
/// Only public functions with this prefix will be added to the list of integration tests.
/// This ensures we only capture actual test functions and not helper functions or comments.
///
/// # Arguments
///
/// * `line` - A line of source code to analyze
///
/// # Returns
///
/// `Some(function_name)` if the line contains a public test function definition, `None` otherwise.
fn parse_test_function_name(line: &str) -> Option<String> {
    let s = line.trim();

    // Skip comments (both single-line and multi-line starts)
    // FIXME: this technically could match with fn names on /* */ blocks
    // but should be good enough for now
    if s.is_empty() || s.starts_with("//") || s.starts_with("/*") {
        return None;
    }

    let tokens: Vec<&str> = s.split_whitespace().collect();
    // Look for public function patterns
    let fn_pos = if tokens[0] == "pub" && tokens[1] == "async" && tokens[2] == "fn" {
        2 // pub async fn 
    } else if tokens[0] == "pub" && tokens[1] == "fn" {
        1 // pub fn 
    } else {
        return None;
    };

    let name_token = tokens.get(fn_pos + 1)?;
    // Extract only valid identifier characters from the name token
    // This stops at '(' for parameters, '<' for generics, etc.
    let ident: String = name_token
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();

    // Check if the identifier starts with "test_"
    let after_prefix = ident.strip_prefix(TEST_PREFIX)?;

    // Ensure there's actually something after the prefix (not just "test_")
    if after_prefix.is_empty() {
        return None;
    }

    // Return the full function name
    Some(format!("{TEST_PREFIX}{after_prefix}"))
}

/// Generates integration test wrappers with individual `#[tokio::test]` functions.
///
/// This function creates a complete Rust source file containing individual tokio test
/// functions that wrap each discovered integration test. Each generated test function
/// handles the setup of `ClientConfig` and calls the original test function.
///
/// # Arguments
///
/// * `test_cases` - Slice of test case metadata to generate wrappers for
///
/// # Returns
///
/// A complete Rust source file as a `String` ready to be written to `OUT_DIR`.
///
/// # Generated Code Structure
///
/// ```rust
/// // File header and imports
/// use anyhow::Result;
///
/// use crate::tests::config::ClientConfig;
/// // ... other imports
///
/// /// Auto-generated tokio test wrapper for my_test
/// #[tokio::test]
/// async fn test_my_test() -> Result<()> {
///     // ClientConfig setup from environment variables
///     let client_config = ClientConfig::new(endpoint, timeout);
///     my_test(client_config).await
/// }
/// ```
///
/// # Environment Variables
///
/// The generated tests respect these environment variables:
/// - `TEST_MIDEN_RPC_ENDPOINT` - RPC endpoint URL (default: localhost)
/// - `TEST_TIMEOUT` - Test timeout in milliseconds (default: 10000)
fn generate_integration_tests(test_cases: &[TestCaseInfo]) -> String {
    let mut result = String::new();

    // Header - use regular comments instead of module docs since this will be included
    result.push_str("// Auto-generated integration tests\n");
    result.push_str("//\n");
    result.push_str(
        "// This module is automatically generated by the build script from test functions\n",
    );
    result.push_str("// named test_*. Do not edit manually.\n\n");

    // Imports
    result.push_str("use anyhow::{anyhow, Result};\n");
    result.push_str("use miden_client_integration_tests::tests::config::ClientConfig;\n");
    result.push_str("use miden_client::rpc::Endpoint;\n");
    result.push_str("use url::Url;\n");

    // Collect unique imports for test modules
    let mut modules = HashSet::new();
    for test_case in test_cases {
        let module_name = &test_case.category;
        modules.insert(module_name);
    }

    for module in modules {
        result.push_str(&format!("use miden_client_integration_tests::tests::{}::*;\n", module));
    }

    result.push('\n');

    // Generate tokio test wrappers for each test case
    for test_case in test_cases {
        // Strip test prefix
        // SAFETY: ok to unwrap here because we collected these names based on the fact they
        // had a "test_" prefix
        let test_fn_name = test_case.function_name.strip_prefix(TEST_PREFIX).unwrap().to_string();

        result.push_str(&format!(
            "/// Auto-generated tokio test wrapper for {}\n",
            test_case.function_name
        ));
        result.push_str("#[tokio::test]\n");
        result.push_str(&format!("async fn {}() -> Result<()> {{\n", test_fn_name));
        result.push_str("    // Use default test configuration\n");
        result.push_str("    let endpoint_url = std::env::var(\"TEST_MIDEN_RPC_ENDPOINT\")\n");
        result.push_str("        .unwrap_or_else(|_| Endpoint::localhost().to_string());\n");
        result.push_str(
            "    let url = Url::parse(&endpoint_url).map_err(|_| anyhow!(\"Invalid RPC endpoint URL\"))?;\n",
        );
        result.push_str("    let host = url\n");
        result.push_str("        .host_str()\n");
        result
            .push_str("        .ok_or_else(|| anyhow!(\"RPC endpoint URL is missing a host\"))?\n");
        result.push_str("        .to_string();\n");
        result.push_str(
            "    let port = url.port().ok_or_else(|| anyhow!(\"RPC endpoint URL is missing a port\"))?;\n",
        );
        result.push_str(
            "    let endpoint = Endpoint::new(url.scheme().to_string(), host, Some(port));\n",
        );
        result.push_str("    let timeout = std::env::var(\"TEST_TIMEOUT\")\n");
        result.push_str("        .unwrap_or_else(|_| \"10000\".to_string())\n");
        result.push_str("        .parse::<u64>()\n");
        result.push_str("        .map_err(|_| anyhow!(\"Invalid timeout value\"))?;\n");
        result.push_str("        \n");
        result.push_str("    let client_config = ClientConfig::new(endpoint, timeout);\n");
        result.push_str(&format!("    {}(client_config).await\n", test_case.function_name));
        result.push_str("}\n\n");
    }

    result
}

/// Generates programmatic test access via `get_all_tests()` function.
///
/// This function creates a Rust source file containing the `get_all_tests()` function
/// that returns a `Vec<TestCase>` for programmatic access to all discovered integration tests.
/// This allows the main application to enumerate and execute tests dynamically.
///
/// # Arguments
///
/// * `test_cases` - Slice of test case metadata to include in the vector
///
/// # Returns
///
/// A complete Rust source file as a `String` ready to be written to `OUT_DIR`.
///
/// # Generated Code Structure
///
/// ```rust
/// use super::{TestCase, TestCategory};
/// use crate::tests::client::*;
/// // ... other module imports
///
/// /// Returns all available test cases organized by category.
/// pub fn get_all_tests() -> Vec<TestCase> {
///     vec![
///         TestCase::new("test_name", TestCategory::Client, test_function),
///         // ... more test cases
///     ]
/// }
/// ```
///
/// # Test Categories
///
/// Categories are automatically derived from file names and converted to PascalCase:
/// - `client.rs` → `TestCategory::Client`
/// - `swap_transaction.rs` → `TestCategory::SwapTransaction`
/// - `custom_transaction.rs` → `TestCategory::CustomTransaction`
fn generate_test_case_vector(test_cases: &[TestCaseInfo]) -> String {
    let mut result = String::new();

    // Header - use regular comments instead of module docs since this will be included
    result.push_str("// Auto-generated test cases module\n");
    result.push_str("//\n");
    result.push_str(
        "// This module is automatically generated by the build script from test functions\n",
    );
    result.push_str("// named test_*. Do not edit manually.\n\n");

    // Imports
    result.push_str("use super::{TestCase, TestCategory};\n");

    // Collect unique imports
    let mut modules = HashSet::new();
    for test_case in test_cases {
        let module_name = &test_case.category;
        modules.insert(module_name);
    }

    for module in modules {
        result.push_str(&format!("use crate::tests::{}::*;\n", module));
    }

    // Function header
    result.push_str("\n/// Returns all available test cases organized by category.\n");
    result.push_str("///\n");
    result.push_str("/// This function is auto-generated from test functions named test_*.\n");
    result.push_str(
        "/// The test cases are automatically discovered by scanning the test modules.\n",
    );
    result.push_str("pub fn get_all_tests() -> Vec<TestCase> {\n");
    result.push_str("    vec![\n");

    // Test cases
    for test_case in test_cases {
        let category_variant =
            format!("TestCategory::{}", snake_case_to_pascal_case(&test_case.category));

        result.push_str(&format!(
            "        TestCase::new(\"{}\", {}, {}),\n",
            test_case.name, category_variant, test_case.function_name
        ));
    }

    result.push_str("    ]\n");
    result.push_str("}\n");

    result
}

/// Converts a snake_case string to PascalCase.
///
/// This utility function is used to convert file names (which are in snake_case)
/// to enum variant names for `TestCategory` (which should be in PascalCase).
///
/// # Arguments
///
/// * `snake_str` - A string in snake_case format
///
/// # Returns
///
/// The input string converted to PascalCase.
///
/// # Examples
///
/// ```
/// assert_eq!(snake_case_to_pascal_case("client"), "Client");
/// assert_eq!(snake_case_to_pascal_case("swap_transaction"), "SwapTransaction");
/// assert_eq!(snake_case_to_pascal_case("custom_transaction"), "CustomTransaction");
/// assert_eq!(snake_case_to_pascal_case("network_transaction"), "NetworkTransaction");
/// ```
///
/// # Algorithm
///
/// 1. Split the input string by underscores
/// 2. Capitalize the first character of each word
/// 3. Join all words together without separators
fn snake_case_to_pascal_case(snake_str: &str) -> String {
    snake_str
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
