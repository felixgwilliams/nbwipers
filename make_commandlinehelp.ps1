$OutputEncoding = [console]::InputEncoding = [console]::OutputEncoding = New-Object System.Text.UTF8Encoding
cargo run -q --features=markdown-help -- --markdown-help check | Set-Content -encoding utf8  CommandLineHelp.md
