(function() {

    const CLIENT_ID = 'marketplace-admin';
    const OAUTH_BASE = '/api/v1/core/oauth';
    const LOGIN_PATH = '/admin/login';
    const DEFAULT_REDIRECT = '/admin/';

    const errorDiv = document.getElementById('error');
    const loadingSection = document.getElementById('loading');
    const retrySection = document.getElementById('retry');

    const generateRandomString = (length) => {
        const array = new Uint8Array(length);
        crypto.getRandomValues(array);
        return Array.from(array, (b) => b.toString(36).padStart(2, '0')).join('').slice(0, length);
    };

    const generateCodeChallenge = async (verifier) => {
        const encoder = new TextEncoder();
        const data = encoder.encode(verifier);
        const digest = await crypto.subtle.digest('SHA-256', data);
        return btoa(String.fromCharCode(...new Uint8Array(digest)))
            .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    };

    const clearAccessToken = async () => {
        try { await fetch('/api/public/auth/session', { method: 'DELETE' }); } catch {}
        document.cookie = 'access_token=; path=/; max-age=0; SameSite=Lax' +
            (window.location.protocol === 'https:' ? '; Secure' : '');
    };

    const showError = async (msg) => {
        await clearAccessToken();
        errorDiv.textContent = msg;
        errorDiv.style.display = 'block';
        loadingSection.style.display = 'none';
        retrySection.style.display = 'block';
    };

    const handleCallback = async () => {
        const params = new URLSearchParams(window.location.search);
        const code = params.get('code');
        const error = params.get('error');
        const errorDesc = params.get('error_description');

        if (error) {
            showError(errorDesc || error);
            return true;
        }

        if (!code) return false;

        const codeVerifier = sessionStorage.getItem('pkce_code_verifier');
        const redirectAfterLogin = sessionStorage.getItem('login_redirect') || DEFAULT_REDIRECT;

        if (!codeVerifier) {
            window.history.replaceState({}, '', LOGIN_PATH);
            startOAuthFlow();
            return true;
        }

        try {
            const tokenResponse = await fetch(OAUTH_BASE + '/token', {
                method: 'POST',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: new URLSearchParams({
                    grant_type: 'authorization_code',
                    code: code,
                    redirect_uri: window.location.origin + LOGIN_PATH,
                    code_verifier: codeVerifier,
                    client_id: CLIENT_ID
                })
            });

            const tokenData = await tokenResponse.json();

            if (!tokenResponse.ok) {
                throw new Error(tokenData.error_description || tokenData.error || 'Token exchange failed');
            }

            const maxAge = tokenData.expires_in || 3600;
            document.cookie = 'access_token=' + tokenData.access_token +
                '; path=/; max-age=' + maxAge + '; SameSite=Lax' +
                (window.location.protocol === 'https:' ? '; Secure' : '');

            try {
                await fetch('/api/public/auth/session', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        access_token: tokenData.access_token,
                        expires_in: maxAge,
                        refresh_token: tokenData.refresh_token || null
                    })
                });
            } catch {}

            sessionStorage.removeItem('pkce_code_verifier');
            sessionStorage.removeItem('pkce_csrf_state');
            sessionStorage.removeItem('login_redirect');

            window.location.href = redirectAfterLogin;
        } catch (err) {
            showError(err.message);
            window.history.replaceState({}, '', LOGIN_PATH);
        }

        return true;
    };

    const clearStaleState = () => {
        sessionStorage.removeItem('pkce_code_verifier');
        sessionStorage.removeItem('pkce_csrf_state');
        sessionStorage.removeItem('login_redirect');
    };

    const startOAuthFlow = async () => {
        await clearAccessToken();
        clearStaleState();
        const codeVerifier = generateRandomString(64);
        const codeChallenge = await generateCodeChallenge(codeVerifier);
        const csrfState = generateRandomString(32);

        sessionStorage.setItem('pkce_code_verifier', codeVerifier);
        sessionStorage.setItem('pkce_csrf_state', csrfState);

        const params = new URLSearchParams(window.location.search);
        const redirectTo = params.get('redirect') || DEFAULT_REDIRECT;
        sessionStorage.setItem('login_redirect', redirectTo);

        const authParams = new URLSearchParams({
            response_type: 'code',
            client_id: CLIENT_ID,
            redirect_uri: window.location.origin + LOGIN_PATH,
            state: csrfState,
            code_challenge: codeChallenge,
            code_challenge_method: 'S256',
            scope: 'user'
        });

        window.location.href = OAUTH_BASE + '/authorize?' + authParams.toString();
    };

    const hasValidAdminToken = () => {
        try {
            const cookie = document.cookie.split('; ').find((c) => c.startsWith('access_token='));
            if (!cookie) return false;
            const token = cookie.split('=').slice(1).join('=');
            const payload = JSON.parse(atob(token.split('.')[1]));
            const scopes = (payload.scope || '').split(' ');
            if (!scopes.includes('user') && !scopes.includes('admin')) return false;
            if (payload.exp && payload.exp * 1000 < Date.now()) return false;
            return true;
        } catch { return false; }
    };

    handleCallback().then(async (wasCallback) => {
        if (wasCallback) return;

        if (hasValidAdminToken()) {
            const params = new URLSearchParams(window.location.search);
            window.location.href = params.get('redirect') || DEFAULT_REDIRECT;
            return;
        }

        await clearAccessToken();
        startOAuthFlow();
    });
})();
