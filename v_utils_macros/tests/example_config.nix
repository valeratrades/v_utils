# Example Nix configuration file for Settings macro
# This file will be evaluated with `nix eval --json --impure --expr 'import <path>'`
# and must return a valid attribute set that can be serialized to JSON

{
  # Simple values
  name = "test_app";
  value = 42;

  # Boolean
  debug = true;

  # Nested structures work too
  # (if your Settings struct has nested fields with #[settings(flatten)])

  # You can use Nix language features like:
  # - String interpolation: "${someVar}"
  # - Conditionals: if condition then value1 else value2
  # - Functions and imports: import ./other.nix
  # - Environment variables (with --impure): builtins.getEnv "HOME"
}
