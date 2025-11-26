# WebAuthn Cross-Device Passkey Setup

## Your Goal: Register Once, Use Everywhere

You want users to:
1. Register a passkey **once**
2. Have it **automatically sync** via browser (Google Password Manager / iCloud Keychain)
3. Use it on **all their devices**

## What We Changed

### Backend Modification (`crates/modules/oauth/src/api/rest/webauthn/register.rs`)

**Before:**
```rust
(StatusCode::OK, headers, Json(challenge)).into_response()
```

**After:**
```rust
// Remove authenticatorAttachment hint to give browser more choice
let mut challenge_json = serde_json::to_value(&challenge)?;
if let Some(public_key) = challenge_json.get_mut("publicKey") {
    if let Some(authenticator_selection) = public_key.get_mut("authenticatorSelection") {
        authenticator_selection.as_object_mut().map(|obj| {
            obj.remove("authenticatorAttachment");
        });
    }
}
(StatusCode::OK, headers, Json(challenge_json)).into_response()
```

**Why:** Removing the `authenticatorAttachment` hint gives the browser more flexibility in choosing authenticators. However, **Chrome on Windows will still likely show Windows Hello first** because it's built-in and always available.

---

## Browser Behavior Reality Check

###  **Chrome on Windows**

**What you'll see during registration:**
1. ✅ Windows Security dialog appears (Windows Hello)
2. ✅ Small link at bottom: "Use a phone, tablet or security key"
3. Clicking that link shows: **Google Password Manager** option

**Why Windows Hello appears first:**
- Chrome prioritizes platform authenticators
- Windows Hello is always available
- Better UX (faster, no network)
- More secure (device-bound TPM)

### **Chrome on Mac**

**What you'll see:**
1. Touch ID prompt (device-bound)
2. OR iCloud Keychain (synced) if signed into iCloud
3. Browser may show both options

### **Mobile Chrome/Safari**

**What you'll see:**
- Fingerprint/Face ID (automatically syncs via Google/Apple)
- These ARE synced passkeys (not device-bound)

---

## How to Ensure Synced Passkeys

### **Method 1: User Must Click "More Options"** (Current Reality)

**Registration Flow:**
1. User fills in form
2. Windows Security appears
3. **User must click**: "Use a phone, tablet or security key"
4. **User must select**: "Save to Google Password Manager"
5. ✅ Passkey syncs to all devices

**Problem:** Extra clicks, confusing UX

### **Method 2: UI Guidance** (Recommended)

Add clear instructions in your registration UI:

```tsx
<div className="passkey-guidance">
  📱 <strong>Want to use this passkey on all your devices?</strong>

  When prompted:
  1. Click "Use a phone, tablet or security key"
  2. Select "Save to Google Password Manager" (Chrome) or use Safari (auto-syncs)
  3. Your passkey will work on all your devices!

  ⚠️ If you choose "Windows Security", the passkey only works on this device.
</div>
```

### **Method 3: Mobile-First Registration** (Best UX)

**Strategy:** Encourage users to register on mobile first

**Why it works:**
- Mobile Chrome/Safari default to synced passkeys
- No Windows Hello to compete with
- Automatically available on desktop after sync

**UI Approach:**
```
"Register on your phone for the best experience"
[Show QR code] → Leads to registration page
```

---

## Testing Your Setup

### **Test 1: Verify Synced Passkey**

1. **Clear existing passkeys:**
   ```bash
   just query "DELETE FROM webauthn_credentials"
   ```

2. **Register on Desktop (Chrome):**
   - Go to registration page
   - Fill in form
   - When Windows Security appears, click "Use a phone, tablet or security key"
   - Select "Save to Google Password Manager"

3. **Check transports in database:**
   ```bash
   just query "SELECT display_name, transports FROM webauthn_credentials"
   ```

   Expected: `["hybrid"]` or `["internal", "hybrid"]`

4. **Test on Mobile:**
   - Open Chrome on phone (same Google account)
   - Navigate to login page
   - Enter email
   - ✅ Should see your passkey available!

### **Test 2: Multiple Devices**

Register on 3 devices to verify sync:
- Desktop Chrome (Windows)
- Mobile Chrome (Android)
- Desktop Chrome (Mac/Linux)

All should see the same passkey after registration on any device.

---

## Common Issues

### "I only see Windows Hello, no Google Password Manager option"

**Check:**
1. Signed into Chrome? (Click profile icon in top-right)
2. Password Manager enabled? `chrome://settings/passwords`
3. Sync enabled? `chrome://settings/syncSetup`

### "Passkey registered but doesn't sync"

**Cause:** You chose Windows Hello instead of Google Password Manager

**Solution:**
```bash
# Delete Windows Hello passkey
just query "DELETE FROM webauthn_credentials WHERE transports = '[\"internal\"]'"

# Re-register with Google Password Manager
```

### "Can't find 'More options' link"

**Location:** At the bottom of the Windows Security dialog

**Alternative:** Press `Esc` to cancel Windows Hello, browser may show other options

---

## Advanced: Force Synced Passkeys Only

If you want to **completely disable device-bound passkeys**, you'd need to:

1. **Detect transports after registration**
2. **Reject device-bound passkeys**:
   ```rust
   if transports == vec!["internal"] {
       return Err("Please use Google Password Manager or iCloud Keychain for cross-device support");
   }
   ```

3. **Add UI messaging** explaining why

**Trade-off:** Better cross-device UX, but more friction during registration.

---

## Recommendation

**Best approach for your use case:**

1. ✅ Keep current backend (allows both types)
2. ✅ Add clear UI guidance during registration
3. ✅ Encourage mobile-first registration
4. ✅ Support multiple passkeys per user (desktop + mobile devices)

This balances UX, security, and cross-device convenience.

---

## Current Implementation Status

✅ **Backend supports synced passkeys**
✅ **Multiple passkeys per user supported**
✅ **Transports stored and used in authentication**
✅ **authenticatorAttachment hint removed**

⚠️ **Browser still shows Windows Hello first** (expected behavior, not a bug)

🎯 **Next step: Add UI guidance for users**
