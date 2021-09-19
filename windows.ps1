cargo build --release
Compress-Archive -Path "target/release/evm.exe", "target/release/SDL2.dll", "C:/Windows/System32/vcruntime140.dll" -DestinationPath "artifacts/Windows-BasePC2.zip" -Update