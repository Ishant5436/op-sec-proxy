# VHS Tape for OP Security Proxy Demo
Output "assets/op_sec_proxy_demo.gif"
Output "assets/op_sec_proxy_demo.mp4"

Set FontSize 14
Set Width 1200
Set Height 700
Set Padding 20
Set Theme "TokyoNight"
Set TypingSpeed 25ms

Type "cargo test --quiet"
Enter
Sleep 3s

Type "node --test sdk/tests/provider.test.js"
Enter
Sleep 3s

Type "cargo run -- --help"
Enter
Sleep 3s

Sleep 2s
