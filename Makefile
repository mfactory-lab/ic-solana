
.PHONY: build
.SILENT: build
build:
	dfx canister create solana_rpc
	dfx build

.PHONY: start
start:
	RUST_BACKTRACE=1 dfx start --clean

.PHONY: test
test:
	dfx canister create --all
	dfx deploy solana_rpc --argument "(record {nodesInSubnet = opt 28})" --mode reinstall -y
	dfx canister call solana_rpc sol_getBalance '("AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY")'
	#dfx canister logs solana_rpc
#	dfx stop
#	echo "BUILD_ENV is ${BUILD_ENV}"
##	bash ./pre_deploy.sh
##	echo "Pre deploy script succeeded"
#	npm install
#	rm -fr .dfx
#	dfx start --clean --background
#	dfx canister create --all
#	dfx deploy internet_identity --argument '(null)'
#	dfx canister create vetkd_system_api --specified-id s55qq-oqaaa-aaaaa-aaakq-cai
#	dfx deploy vetkd_system_api
#	dfx deploy encrypted_notes_${BUILD_ENV}
#	dfx generate encrypted_notes_${BUILD_ENV}
#	dfx deploy www
#	echo "Deployment succeeded"
#	echo "Start testing..."
#	dfx canister call encrypted_notes_${BUILD_ENV} whoami
#	sh test_whoami.sh
#	echo "ENCRYPTED NOTES E2E TESTS PASSED"

#.PHONY: test
#test:
#	#dfx stop
#	#bash ./pre_deploy.sh
#	#echo "Pre deploy script succeeded"
##	pnpm install
#	rm -fr .dfx
#	#dfx start --clean --background
#	#dfx deploy internet_identity --argument '(null)'
#	#dfx canister create system_api --specified-id s55qq-oqaaa-aaaaa-aaakq-cai
#	#dfx deploy system_api
#	dfx deploy solana_rpc
#	dfx generate solana_rpc
#	#dfx deploy www
#	echo "Deployment succeeded"
#	echo "Start testing..."
#	dfx canister call solana_rpc send_tx
#	dfx canister call solana_rpc send_tx
#	#sh test_whoami.sh
#	echo "TESTS PASSED"

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
