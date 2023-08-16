# vault-fake-remote

Fake Koofr API server.

## Run

```sh
cargo run --example fake_remote

curl -v -k https://127.0.0.1:3443/health

curl -v -k 'https://127.0.0.1:3443/oauth2/auth?redirect_uri=http://localhost:5173&state=dummy'
curl -v -k 'https://127.0.0.1:3443/oauth2/token' -X POST -d 'grant_type=authorization_code&code=fa4ac52a-273c-4e99-ac4c-6147e4c702c1'
curl -v -k 'https://127.0.0.1:3443/oauth2/token' -X POST -d 'grant_type=refresh_token&refresh_token=b8d19847-dfc3-4301-9e5c-86e617141473'

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" https://127.0.0.1:3443/api/v2.1/user

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" https://127.0.0.1:3443/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" 'https://127.0.0.1:3443/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1/bundle?path=/'

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" 'https://127.0.0.1:3443/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1/files/info?path=/'

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" 'https://127.0.0.1:3443/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1/files/folder?path=/' -X POST -H "Content-Type: application/json" -d '{
  "name": "My safe box"
}'

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" 'https://127.0.0.1:3443/content/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1/files/put?path=/&filename=Cargo.toml&info=true' --data-binary "@Cargo.toml"

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" 'https://127.0.0.1:3443/content/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1/files/get?path=/Cargo.toml'

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" 'https://127.0.0.1:3443/content/api/v2.1/mounts/9fd62581-3bad-478a-702b-01937d2bf7f1/files/listrecursive?path=/'

curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" https://127.0.0.1:3443/api/v2.1/vault/repos -X POST -H "Content-Type: application/json" -d '{
  "mountId": "9fd62581-3bad-478a-702b-01937d2bf7f1",
  "path": "/My safe box",
  "salt": "salt",
  "passwordValidator": "a8668309-60f9-40f1-9a4c-0d1de0ff5852",
  "passwordValidatorEncrypted": "v2:UkNMT05FAADWjQahYq7E1ij2zegBBHbFuDbGIHAvdpym3P4eW2CPQcWhcTuAz4YGLAwRQzj2PoP4vwS2hAEwFwqMlFsWTgLMQ2ONzdNJK4d3kaVw"
}'
curl -v -k -H "Authorization: Bearer f1fed68a-6b5c-4067-928e-40ed48dd2589" https://127.0.0.1:3443/api/v2.1/vault/repos
```
