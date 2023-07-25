use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_mobile::User;

use crate::helpers::with_user;

#[test]
fn test_user() {
    with_user(|fixture| {
        async move {
            let user = fixture
                .wait(|v, cb| v.user_subscribe(cb), |v, id| v.user_data(id))
                .await;

            assert_eq!(
                user,
                User {
                    id: fixture.user_id.clone(),
                    first_name: "Vault".into(),
                    last_name: "Test".into(),
                    full_name: "Vault Test".into(),
                    email: user.email.clone(),
                }
            );
        }
        .boxed()
    });
}
