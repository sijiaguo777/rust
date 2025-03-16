# Question 1
what is the lifetime of s? Who is the owner of the underlying string with spaces (every object has an owner)?
- s is temporary value is freed at the end of statement.
- Owner of string with space is the temporary String returned by ret_string(). Since this String is not assigned to a variable in main, it is immediately dropped at the end of the expression.



cargo build --release
   Compiling problems v0.1.0 (/Users/guosijia/Desktop/rust/sijia-guo/problems)

error: free function without a body
 --> src/main.rs:1:1
  |
1 | fn trim(&self) -> &str;
  | ^^^^^^^^^^^^^^^^^^^^^^-
  |                       |
  |                       help: provide a definition for the function: `{ <body> }`

error: `self` parameter is only allowed in associated functions
 --> src/main.rs:1:9
  |
1 | fn trim(&self) -> &str;
  |         ^^^^^ not semantically valid as function parameter
  |
  = note: associated functions are those in `impl` or `trait` definitions

error[E0716]: temporary value dropped while borrowed
 --> src/main.rs:8:13
  |
8 |     let s = ret_string().trim();
  |             ^^^^^^^^^^^^       - temporary value is freed at the end of this statement
  |             |
  |             creates a temporary value which is freed while still in use
9 |     assert_eq!(s, "A String object");
  |     -------------------------------- borrow later used here
  |
help: consider using a `let` binding to create a longer lived value
  |
8 ~     let binding = ret_string();
9 ~     let s = binding.trim();
  |

For more information about this error, try `rustc --explain E0716`.
error: could not compile `problems` (bin "problems") due to 3 previous errors
