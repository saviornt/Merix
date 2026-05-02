# Rust llama-cpp-2 compile using the :latest (as of May 1, 2026)

This is a small guide for compiling llama.cpp when using your development stack is:

- Windows 11
- Visual Studio Community Edition 2026
- Visual Studio Code (Main IDE)
- MSVC Build Tools for x64/x86 (Latest)
- NVIDIA CUDA 13.2
- Rust: version 1.95
- llama-cpp-2: version 0.1.146 with cuda feature

## Requirements

### Application / Package Installation Links

>**Visual Studio Community Edition 2026** can be found [here.](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=Community&channel=Stable&version=VS18&source=VSLandingPage&passive=false&cid=2500)

>**Visual Studio Code** can be found [here.](https://code.visualstudio.com/download)

> **NVIDIA Developer Toolkit** (CUDA Toolkit 13.2 Update 1) is located [here.](https://developer.nvidia.com/cuda-downloads)

### VS 2026 Installation Details

When installing VS 2026, select the following options:

- MSVC Build Tools for x64/x86 (Latest)
- C++ CMake tools for Windows
- Windows 11 SDK
- MSVC v143 - VS 2022 C++ x64/x86 build tools
- .NET 8.0 Runtime
- .NET Framework 4.8.1 SDK (UE5.7 game development)
- MSBuild support for LLVM (clang-cl) toolset
- C++ Clang Compiler for Windows
- Git for Windows (of course)

### NVIDIA Developer Toolkit Details

After installing the CUDA Toolkit 13.2 Update 1, for whatever reason NVIDIA decided not to include it into the PATH. So...

1. WinKey -> Type `Environment variables` -> Press enter.
2. Press `Environment Variables`
3. Double-click on `Path`.
4. Click on `New`.
5. Add this entry: `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.2\bin`.
6. Click `Ok`.
7. Click on `New`.
8. Add these entries:

    - "Variable name": `CUDACXX`, "Variable value": `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.2\bin\nvcc.exe`
    - "Variable name": `CUDAToolkit_ROOT`, "Variable value": `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.2`
    - "Variable name": `CudaToolkitDir`, "Variable value": `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.2`
    - "Variable name": `UV_TORCH_BACKEND`, "Variable value": `cu130` (only if you're working with Pytorch version 2.11.0 with CUDA 13.0)

9. Click `Ok`, `Ok` & restart your computer - or at least, log out and log back in.

### Install Ninja builder

Open powershell, use one of these:

> `winget install --id Ninja-build.Ninja -e`

or

> `choco install ninja`

### Verification

After everything has been installed, open the `x64 Native Tools Prompt for VS` and type the following commands:

- `nvcc --version`
- `ninja --version`
- `cl`

You should get outputs for all three. If not, something is tragecially wrong. Contact your IT Support representative.

### Configure Cargo

1. On Windows, open up a File Explorer and go to the address bar. Type in `%USERPROFILE%\.cargo` and press enter.
2. Either create or edit the `config.toml`:

```toml
[env]
CMAKE_GENERATOR = "Ninja"
CMAKE_BUILD_PARALLEL_LEVEL = "8"
CUDACXX = "C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v13.2/bin/nvcc.exe"
```

*(Yes, we added `CUDACXX` to the environment variables, but cargo doesn't pick it up. Don't ask me why, I don't develop it)*

### Add llama-cpp-rs to your project

Easy, just add the following to your `Cargo.toml`:

```toml
llama-cpp-2 = { version = "0.1", features = ["cuda"] }
```

### Building with Cargo

This is where it gets annoying, we have to use the `x64 Native Tools Prompt for VS` command prompt to build/check.

1. `cd /d {X:\WHERE-EVER-YOUR-PROJECT-IS}` (take out the `{}` and add the actual directory). Press enter.
2. `cargo clean` -> `cargo check --workspace`
3. It *should* build correctly.

### Unverified Developer Prompt for VS Code

**I haven't tested this, yet... but it should work *in theory*:**

To get the x64 Native Tools Command Prompt inside of VS Code (activating `vcvars64.bat` within the terminal), use the following inside of your project's `.vscode/settings.json` file:

```json
{
    "terminal.integrated.profiles.windows": {
    "MSVC Dev Prompt": {
      "path": "C:\\Windows\\System32\\cmd.exe",
      "args": [
        "/k",
        "\"C:\\Program Files\\Microsoft Visual Studio\\18\\Community\\VC\\Auxiliary\\Build\\vcvars64.bat\""
      ]
    }
  },
  "terminal.integrated.defaultProfile.windows": "MSVC Dev Prompt"
}
```
