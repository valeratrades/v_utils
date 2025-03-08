# Dev
## Testing
Some tests are hidden behind `slow_tests` feature-flag, so before release run tests with `-F slow_tests`
```sh
cargo nextest run -F slow_tests
```
