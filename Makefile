CANISTER_ID=solana_rpc

.PHONY: deploy
deploy:
	./deploy.sh

.PHONY: start
start:
	RUST_BACKTRACE=1 dfx start --clean

.PHONY: test
test:
	./deploy.sh
	dfx canister call ${CANISTER_ID} sol_getBalance '("AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY")'
#dfx canister logs ${CANISTER}
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
#	echo "TESTS PASSED"

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
