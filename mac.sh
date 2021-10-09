cargo build --release
install_name_tool -add_rpath @executable_path target/release/evm
cp target/release/libSDL2-2.0.dylib artifacts/BasePC\ 2.0.app/Contents/MacOS/libSDL2-2.0.dylib

create-dmg --volname "BasePC 2.0" "artifacts/BasePC 2.0.app"

mv "BasePC 2.0 0.01.dmg" "artifacts/MacOS-BasePC2.dmg"
