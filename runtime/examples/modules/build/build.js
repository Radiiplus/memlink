#!/usr/bin/env node

const { execSync, spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

const MODULES_DIR = path.join(__dirname, "..");
const BUILD_DIR = path.join(__dirname);
const SRC_DIR = MODULES_DIR;

const modules = ["math", "string", "crypto", "echo"];

const isWindows = os.platform() === "win32";
const isLinux = os.platform() === "linux";

function log(message, type = "info") {
  const prefix =
    {
      info: "ℹ",
      success: "✓",
      error: "✗",
      warn: "⚠",
    }[type] || "ℹ";
  console.log(`${prefix} ${message}`);
}

function runCommand(command, args, options = {}) {
  try {
    const result = spawnSync(command, args, {
      stdio: "inherit",
      shell: true,
      ...options,
    });
    return result.status === 0;
  } catch (error) {
    log(`Command failed: ${error.message}`, "error");
    return false;
  }
}

function runWslCommand(command) {
  try {
    const result = spawnSync("wsl", ["bash", "-c", command], {
      stdio: "pipe",
      shell: false,
    });
    return result.status === 0;
  } catch (error) {
    log(`WSL command failed: ${error.message}`);
    return false;
  }
}

function findCompiler() {
  const compilers = {
    msvc: null,
    gcc: null,
    wsl: null,
  };

  try {
    execSync("cl.exe", { stdio: "ignore" });
    compilers.msvc = "cl.exe";
    log("Found MSVC compiler (cl.exe)", "success");
  } catch (e) {
    log("MSVC (cl.exe) not found");
  }

  try {
    execSync("gcc --version", { stdio: "ignore" });
    compilers.gcc = "gcc";
    log("Found MinGW gcc", "success");
  } catch (e) {
    log("MinGW gcc not found");
  }

  try {
    execSync("wsl echo test", { stdio: "ignore" });
    compilers.wsl = "wsl";
    log("Found WSL", "success");
  } catch (e) {
    log("WSL not available");
  }

  return compilers;
}

function buildWindowsDll(compiler) {
  log(`Building Windows DLLs with ${compiler}...`);

  for (const module of modules) {
    const srcFile = path.join(SRC_DIR, `${module}.c`);
    const outFile = path.join(MODULES_DIR, `${module}.dll`);

    if (!fs.existsSync(srcFile)) {
      log(`Source file not found: ${srcFile}`);
      continue;
    }

    let success = false;

    if (compiler === "cl.exe") {
      success = runCommand("cl.exe", [
        "/LD",
        "/Fe:" + outFile,
        srcFile,
        "/link",
        "/OUT:" + outFile,
      ]);
    } else if (compiler === "gcc") {
      success = runCommand("gcc", [
        "-shared",
        "-o",
        outFile,
        srcFile,
        "-Wl,--add-stdcall-alias",
      ]);
    }

    if (success) {
      log(`Built: ${outFile}`, "success");
    } else {
      log(`Failed to build: ${outFile}`);
    }
  }
}

function buildLinuxSo() {
  log("Building Linux shared objects with WSL gcc...");

  const wslModulesDir = execSync('wsl wslpath -u "' + SRC_DIR + '"')
    .toString()
    .trim();

  for (const module of modules) {
    const srcFile = path.join(wslModulesDir, `${module}.c`).replace(/\\/g, "/");
    const outFile = path
      .join(wslModulesDir, `lib${module}.so`)
      .replace(/\\/g, "/");

    const command = `gcc -shared -fPIC -O2 -o "${outFile}" "${srcFile}"`;

    if (runWslCommand(command)) {
      log(`Built: lib${module}.so`, "success");
    } else {
      log(`Failed to build: lib${module}.so`);
    }
  }

  log("Copying built modules to examples/modules/...");
  const wslParentDir = path.dirname(wslModulesDir).replace(/\\/g, "/");
  for (const module of modules) {
    const srcFile = `${wslModulesDir}/lib${module}.so`;
    const destFile = `${wslParentDir}/modules/lib${module}.so`;
    runWslCommand(`cp "${srcFile}" "${destFile}"`);
  }
}

function buildMacosDylib() {
  log("Building macOS dynamic libraries...");

  for (const module of modules) {
    const srcFile = path.join(SRC_DIR, `${module}.c`);
    const outFile = path.join(MODULES_DIR, `lib${module}.dylib`);

    if (!fs.existsSync(srcFile)) {
      log(`Source file not found: ${srcFile}`);
      continue;
    }

    const success = runCommand("cc", [
      "-shared",
      "-fPIC",
      "-O2",
      "-o",
      outFile,
      srcFile,
    ]);

    if (success) {
      log(`Built: ${outFile}`, "success");
    } else {
      log(`Failed to build: ${outFile}`);
    }
  }
}

function clean() {
  log("Cleaning built files...");

  const extensions = [".dll", ".so", ".dylib", ".exp", ".lib", ".obj"];
  const prefixes = ["", "lib"];

  for (const module of modules) {
    for (const ext of extensions) {
      for (const prefix of prefixes) {
        const file = path.join(MODULES_DIR, `${prefix}${module}${ext}`);
        if (fs.existsSync(file)) {
          fs.unlinkSync(file);
          log(`Removed: ${file}`, "info");
        }
      }
    }
  }
}

function showHelp() {
  console.log(`
Memlink Runtime Module Builder

Usage: node build.js [options]

Options:
  --all        Build for all platforms (Windows DLL + Linux SO via WSL)
  --windows    Build only Windows DLL
  --linux      Build only Linux SO (via WSL)
  --macos      Build only macOS dylib
  --clean      Remove built binaries
  --help       Show this help message

Examples:
  node build.js              # Build for current platform
  node build.js --all        # Build for Windows and Linux
  node build.js --clean      # Clean built files
`);
}

function main() {
  const args = process.argv.slice(2);

  if (args.includes("--help") || args.includes("-h")) {
    showHelp();
    return;
  }

  if (args.includes("--clean")) {
    clean();
    return;
  }

  const compilers = findCompiler();

  const buildAll = args.includes("--all");
  const buildWindows =
    args.includes("--windows") ||
    (!args.includes("--linux") && !args.includes("--macos"));
  const buildLinux = args.includes("--linux") || buildAll;
  const buildMacos = args.includes("--macos");

  if (buildWindows) {
    if (compilers.msvc || compilers.gcc) {
      buildWindowsDll(compilers.msvc || compilers.gcc);
    } else {
      log("No Windows compiler found. Install MSVC or MinGW.");
    }
  }

  if (buildLinux) {
    if (compilers.wsl) {
      buildLinuxSo();
    } else {
      log("WSL not available for Linux build");
    }
  }

  if (buildMacos) {
    buildMacosDylib();
  }

  log("Build complete!", "success");
}

main();
