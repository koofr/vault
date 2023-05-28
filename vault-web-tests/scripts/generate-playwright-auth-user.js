const fs = require('fs');

function generate(envName, fileName, defaultVaultOAuth2TokenJSON) {
  let vaultOAuth2TokenJSON = process.env[envName];

  if (!vaultOAuth2TokenJSON || vaultOAuth2TokenJSON.length === 0) {
    if (defaultVaultOAuth2TokenJSON == null) {
      throw new Error(`Missing env variable ${envName}`);
    } else {
      vaultOAuth2TokenJSON = defaultVaultOAuth2TokenJSON;
    }
  }

  const vaultOAuth2Token = JSON.parse(vaultOAuth2TokenJSON);

  if (
    !vaultOAuth2Token.refresh_token ||
    vaultOAuth2Token.refresh_token.length === 0
  ) {
    throw new Error(`Invlaid ${envName} (missing refresh_token)`);
  }

  fs.writeFileSync(
    `playwright/.auth/${fileName}`,
    JSON.stringify(
      {
        cookies: [],
        origins: [
          {
            origin: 'http://localhost:5173',
            localStorage: [
              {
                name: 'vaultOAuth2Token',
                value: vaultOAuth2TokenJSON,
              },
            ],
          },
        ],
      },
      null,
      2
    )
  );
}

generate(
  'VAULT_OAUTH2_TOKEN',
  'user.json',
  `{"access_token":"","refresh_token":"a126768a-ce0b-4b93-8a9b-809f02f4c000","expires_at":0}`
);

// for (let i = 0; i < 8; i++) {
//   generate(`VAULT_OAUTH2_TOKEN_${i}`, `user-${i}.json`);
// }
