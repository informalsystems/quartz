# Setup local accounts. This is only for local development and should not be used in production.
import-local-accounts:
	neutrond keys add val1 --keyring-backend test > ./accounts/val1.txt 2>&1
	neutrond keys add val2 --keyring-backend test > ./accounts/val2.txt 2>&1
	neutrond keys add demowallet1 --keyring-backend test > ./accounts/demowallet1.txt 2>&1
	neutrond keys add demowallet2 --keyring-backend test > ./accounts/demowallet2.txt 2>&1
	neutrond keys add demowallet3 --keyring-backend test > ./accounts/demowallet3.txt 2>&1
	neutrond keys add rly1 --keyring-backend test > ./accounts/rly1.txt 2>&1
	neutrond keys add rly2 --keyring-backend test > ./accounts/rly2.txt 2>&1
.PHONY: create-local-accounts