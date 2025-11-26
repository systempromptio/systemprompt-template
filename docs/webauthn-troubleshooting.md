# WebAuthn Troubleshooting Guide

## Google Password Manager Not Appearing

### Prerequisites
1. **Sign into Chrome**: Click your profile icon → Sign in with Google account
2. **Enable Password Manager**:
   - Go to `chrome://settings/passwords`
   - Enable "Offer to save passwords and passkeys"
3. **Verify Google Account**: Go to `chrome://password-manager/passwords` to confirm it's working

### During Registration

**What you'll see:**
- Windows Security dialog appears automatically ✅ (expected)
- Google Password Manager is hidden in "more options" ✅ (expected)

**To access Google Password Manager:**
1. Look for "Use a different passkey" or "More options" link
2. OR Cancel Windows Security dialog
3. Select "Save to Google Password Manager" from the list

### Why Chrome Prefers Windows Hello

Chrome prioritizes platform authenticators because:
- Higher security (device-bound)
- Faster authentication
- Better UX (biometric)

Google Password Manager is for **cross-device convenience**, while Windows Hello is for **device security**.

---

## Synced vs Platform Passkeys

| Type | Storage | Sync | Security | Use Case |
|------|---------|------|----------|----------|
| **Windows Hello** | TPM chip | ❌ No | 🔒 Highest | Single device, max security |
| **Google Password Manager** | Cloud | ✅ Yes | 🔐 High | Multiple devices, convenience |

Both are secure and valid choices!

---

## Forcing Synced Passkeys (Advanced)

If you want to **prioritize** Google Password Manager over Windows Hello, you need to modify the WebAuthn implementation to use conditional UI or hints. This requires code changes to the Rust backend.

See: `/docs/webauthn-advanced-config.md`
