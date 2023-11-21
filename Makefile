
.PHONY: build
.SILENT: build
build:
	dfx canister create app_backend
	dfx build

.PHONY: start
start:
	RUST_BACKTRACE=1 dfx start --clean

.PHONY: test
test:
	#dfx stop
	#bash ./pre_deploy.sh
	#echo "Pre deploy script succeeded"
#	pnpm install
	rm -fr .dfx
	#dfx start --clean --background
	#dfx deploy internet_identity --argument '(null)'
	#dfx canister create system_api --specified-id s55qq-oqaaa-aaaaa-aaakq-cai
	#dfx deploy system_api
	dfx deploy app_backend
	dfx generate app_backend
	#dfx deploy www
	echo "Deployment succeeded"
	echo "Start testing..."
	dfx canister call app_backend send_tx
	dfx canister call app_backend send_tx
	#sh test_whoami.sh
	echo "TESTS PASSED"

.PHONY: test-unit
test-unit:
	bash ./src/backend/test/run_tests.sh
	echo "UNIT TESTS PASSED"

.PHONY: clean
clean:
	rm -rf .dfx
	rm -rf node_modules
	rm -rf src/declarations
#	rm -rf src/frontend/public/build
#	rm -rf src/frontend/src/lib/backend.ts
#	rm -rf src/frontend/src/lib/idlFactory.js
#	rm -rf dfx.json
	cargo clean
