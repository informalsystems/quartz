# Errors while installing SGX on real azure machine

1. Installing quartz - old way

```sh
cargo install --path crates/cli --locked
```

```
error: failed to run custom build command for `mc-sgx-core-sys-types v0.11.0`

Caused by:
  process didn't exit successfully: `/home/azure_enclave/cycles-quartz/target/release/build/mc-sgx-core-sys-types-6f9403c9b261f4c2/build-script-build` (exit status: 101)
  --- stdout
  cargo:rerun-if-changed=/home/azure_enclave/.cargo/registry/src/index.crates.io-6f17d22bba15001f/mc-sgx-core-build-0.11.0/headers/tlibc
  cargo:rerun-if-changed=/home/azure_enclave/.cargo/registry/src/index.crates.io-6f17d22bba15001f/mc-sgx-core-build-0.11.0/headers

  --- stderr
  thread 'main' panicked at /home/azure_enclave/.cargo/registry/src/index.crates.io-6f17d22bba15001f/bindgen-0.66.1/lib.rs:604:31:
  Unable to find libclang: "couldn't find any valid shared libraries matching: ['libclang.so', 'libclang-*.so', 'libclang.so.*', 'libclang-*.so.*'], set the `LIBCLANG_PATH` environment variable to a path where one of these files can be found (invalid: [])"
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
warning: build failed, waiting for other jobs to finish...
error: failed to compile `quartz-rs v0.2.0 (/home/azure_enclave/cycles-quartz/crates/cli)`, intermediate artifacts can be found at `/home/azure_enclave/cycles-quartz/target`.
To reuse those artifacts with a future compilation, set the environment variable `CARGO_TARGET_DIR` to that path.
```

2. Installing quartz - new way

```sh

export LIBCLANG_PATH=/usr/lib/llvm-14/lib

echo 'export LIBCLANG_PATH=/usr/lib/llvm-14/lib' >> ~/.bashrc
source ~/.bashrc


cargo install quartz-rs --locked
```

3. Installing gramine - working now on ubuntu 24

```sh
# For Gramine, let's try to explicitly import the key
curl -fsSL https://packages.gramineproject.io/gramine-keyring.gpg | sudo gpg --import
sudo gpg --export --armor 4B8D8EC2F8BE4647 | sudo apt-key add -

# Update the repository file
echo "deb [trusted=yes] https://packages.gramineproject.io/ noble main" | sudo tee /etc/apt/sources.list.d/gramine.list

# Update and install
sudo apt-get update
sudo apt-get install -y gramine
```

4. Installing dependencies

dcap client fix - using older ubuntu release ? it works

sudo apt install libsgx-dcap-ql libsgx-dcap-default-qpl libsgx-quote-ex

```sh
curl -sSL https://packages.microsoft.com/keys/microsoft.asc | sudo gpg --dearmor -o /usr/share/keyrings/microsoft-archive-keyring.gpg
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/microsoft-archive-keyring.gpg] https://packages.microsoft.com/ubuntu/22.04/prod/ jammy main" | sudo tee /etc/apt/sources.list.d/microsoft-prod.list


sudo apt update
sudo apt install -y az-dcap-client
```

5. neutron
```sh
 neutrond keys add admin --keyring-backend test --recover

> Enter your bip39 mnemonic
garage advice weekend this dose mango sign horse tool torch mosquito repeat sentence valid scheme pull punch need prosper build actor say cancel allow

- address: neutron1zmr7dfc325907tvj8jl2p2p4cx84clk5aflstl
  name: admin
  pubkey: '{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"Ak3alwik1L65ujlKbgQP//JL3LUlJmvA0zn8rt4eeoHG"}'
  type: local
```

6. nodejs


Install nvm:

```sh
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
```

Reload your shell:

```sh
source ~/.bashrc
```
Install Node.js 20.10.0:

```sh
nvm install 20.10.0
```
Set Node.js 20.10.0 as default:

```sh
nvm use 20.10.0
nvm alias default 20.10.0
```
Verify installation:

```bash
node -v
npm -v
```


7. SGX-DCAP-PCCS

If the error is 


```
sudo apt update
sudo apt install --reinstall ca-certificates
sudo update-ca-certificates
```


```sh
sudo apt upgrade libsgx-dcap-default-qpl
```


```
sudo apt-get install sgx-dcap-pccs
```

This only works with focal (22.04) and not ubuntu noble 24.04
```sh
echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
```


Questions to be answered:
Do you want to install PCCS now? (Y/N) :Y
Check proxy server configuration for internet connection... 
Enter your http proxy server address, e.g. http://proxy-server:port (Press ENTER if there is no proxy server) :

Do you want to configure PCCS now? (Y/N) :y
Set HTTPS listening port [8081] (1024-65535) :8081
Set the PCCS service to accept local connections only? [Y] (Y/N) :n
Set your Intel PCS API key (Press ENTER to skip) :
You didn't set Intel PCS API key. You can set it later in config/default.json.  
Choose caching fill method : [LAZY] (LAZY/OFFLINE/REQ) :LAZY
Set PCCS server administrator password: Sunset7-Galaxy@Bridge
Set PCCS server user password: Sunrise8-Violin@Sound

Do you want to generate insecure HTTPS key and cert for PCCS service? [Y] (Y/N) :y
You are about to be asked to enter information that will be incorporated
into your certificate request.
What you are about to enter is what is called a Distinguished Name or a DN.
There are quite a few fields but you can leave some blank
For some fields there will be a default value,
If you enter '.', the field will be left blank.
-----
Country Name (2 letter code) [AU]:CA
State or Province Name (full name) [Some-State]:Ontario
Locality Name (eg, city) []:Toronto
Organization Name (eg, company) [Internet Widgits Pty Ltd]:Informal Systems
Organizational Unit Name (eg, section) []:Cycles
Common Name (e.g. server FQDN or YOUR name) []:azure-sgx
Email Address []:info@cycles.money

Please enter the following 'extra' attributes
to be sent with your certificate request

A challenge password []:Sunshine9
An optional company name []:
Certificate request self-signature ok
subject=C = CA, ST = Ontario, L = Toronto, O = Informal Systems, OU = Cycles, CN = azure-sgx, emailAddress = info@cycles.money
Installing PCCS service ...Created symlink /etc/systemd/system/multi-user.target.wants/pccs.service → /usr/lib/systemd/system/pccs.service.
finished.
Installation completed successfully.
Processing triggers for libc-bin (2.39-0ubuntu8.4) ...


sudo systemctl start pccs

# update /etc/sgx_default_qcnl.conf to config in our repo
sudo cp sgx_default_qcnl.conf /etc/sgx_default_qcnl.conf

# reset pccs
sudo systemctl restart pccs