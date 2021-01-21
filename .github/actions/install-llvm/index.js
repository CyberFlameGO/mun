const core = require('@actions/core');
const exec = require('@actions/exec');
const tc = require('@actions/tool-cache');
const path = require('path');

const isWindows = process.platform == "win32"
const isMacOS = process.platform == "darwin"
const isLinux = process.platform == "linux"

export async function execute(cmd) {
    let myOutput = '';
    let myError = '';
    await exec.exec(cmd, [], {
        listeners: {
        stdout: (data) => {
            myOutput += data.toString().trim();
        },
        stderr: (data) => {
            myError += data.toString().trim();
        }
        }
    });

    if (myError) {
        throw new Error(myError);
    }
    return myOutput;
}

(async () => {
    try {
        if(isLinux) {
            await exec.exec("sudo apt install llvm-8 llvm-8-* liblld-8*");
        } else if(isMacOS) {
            await exec.exec("brew install llvm@8")
            let llvmPath = await execute("brew --prefix llvm@8");
            core.addPath(`${llvmPath}/bin`)   
        } else if(isWindows) {
            const downloadUrl = "https://github.com/mun-lang/llvm-package-windows/releases/download/v8.0.1/llvm-8.0.1-windows-x64-msvc16.7z"
            core.info(`Downloading LLVM from '${downloadUrl}'`)
            const downloadLocation = await tc.downloadTool(downloadUrl);

            core.info("Succesfully downloaded LLVM release, extracting...")
            const llvmPath = path.resolve("llvm");
            const _7zPath = path.join(__dirname, '..', 'externals', '7zr.exe');
            const args = [
                "-bsp1", // output log info
                "x", // extract
                downloadLocation,
                `-o${llvmPath}`
            ]
            const exit = await exec.exec(_7zPath, args);
            if(exit !== 0) {
                throw new Error("Could not extract LLVM and Clang binaries.");
            }

            core.info("Succesfully extracted LLVM release")
            core.addPath(`${llvmPath}/bin`)
            core.exportVariable('LIBCLANG_PATH', `${llvmPath}/bin`)
        } else {
            core.setFailed(`unsupported platform '${process.platform}'`)
        }    
    } catch(error) {
        console.error(error.stack);
        core.setFailed(error.message);
    }
})();
