const fs = require('fs');

const vaultOAuth2TokenJSON = process.env.VAULT_OAUTH2_TOKEN;

if (!vaultOAuth2TokenJSON || vaultOAuth2TokenJSON.length === 0) {
  throw new Error('Missing env variable VAULT_OAUTH2_TOKEN');
}

const vaultOAuth2Token = JSON.parse(vaultOAuth2TokenJSON);

if (
  !vaultOAuth2Token.refresh_token ||
  vaultOAuth2Token.refresh_token.length === 0
) {
  throw new Error('Invlaid VAULT_OAUTH2_TOKEN (missing refresh_token)');
}

fs.writeFileSync(
  'playwright/.auth/user.json',
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
