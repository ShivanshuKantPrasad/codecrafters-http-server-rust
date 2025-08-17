[![progress-banner](https://backend.codecrafters.io/progress/http-server/93400d7c-2427-44f3-8fd7-63298872c41f)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own HTTP server" Challenge](https://app.codecrafters.io/courses/http-server/overview).

[HTTP](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol) is the
protocol that powers the web. In this challenge, you'll build a HTTP/1.1 server
that is capable of serving multiple clients.

Along the way you'll learn about TCP servers,
[HTTP request syntax](https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html),
and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your HTTP server implementation is in `src/main.rs`. Study
and uncomment the relevant code, and push your changes to pass the first stage:

```sh
git add .
git commit -m "pass 1st stage" # any msg
git push origin master
```

Time to move on to the next stage!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.70)` installed locally
1. Run `./your_server.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

# Induction Hack Submission

- Requires rust
- Run with `./your_server.sh --directory public/` to host the files in public folder.

Requirements
[x] Open a TCP socket on a specified port and listen for connections.
[x] Correctly parse at least GET requests: Extract method, requested path, and headers.
[x] Respond with a body – Send a text or HTML body along with correct Content-Length.
[x] Handle concurrent connections – Serve multiple clients at the same time.
[x] Handle multiple requests sequentially without crashing.
[x] Serve static files from a predefined folder (e.g., public/):
[x] Return a 200 response with the file’s content if it exists.
[x] Return a 404 response if the file is not found.

[ ] Support additional methods: POST or HEAD.
[x] Serve a default index.html when / is requested.
[ ] HTTP/1.1 persistent connections — allow multiple requests on the same connection.
[x] Support multiple compression schemes (gzip, deflate, etc.).
