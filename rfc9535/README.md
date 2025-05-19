# Tests for RFC9535

This directory contains tests for the [RFC9535](https://www.rfc-editor.org/info/rfc9535) implementation.

## Usage
Run  `main.rs`.
It will print the test results in the console with the following format:
```
...
Skipping test case: `<name>` because of reason: `reason`
...
Failed tests:

------- <name> -------
<reason>

...

RFC9535 Compliance tests:
Total: 671
Passed: 209
Failed: 462
Skipped: 25 where 18 to fix in 5 issues

```

The results will be saved in the `results.csv` file.

The cases can be filtered using `filtered_cases.json` file. 
The file should contain json array with the test case names that should be filtered out and the reason.
