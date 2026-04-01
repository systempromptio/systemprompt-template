import { apiFetch, apiGet } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

let paddleReady = false;
let paddleConfigured = false;

const loadPaddle = () => {
  const config = document.getElementById('paddle-config');
  if (config) {
    const clientToken = config.getAttribute('data-client-token');
    const environment = config.getAttribute('data-environment');
    if (clientToken) {
      paddleConfigured = true;
      const script = document.createElement('script');
      script.src = 'https://cdn.paddle.com/paddle/v2/paddle.js';
      script.onload = () => {
        if (environment === 'sandbox') Paddle.Environment.set('sandbox');
        Paddle.Initialize({
          token: clientToken,
          eventCallback: (event) => {
            if (event.name === 'checkout.completed') {
              showToast('Subscription activated! Refreshing...', 'success');
              setTimeout(() => window.location.reload(), 2000);
            }
          },
        });
        paddleReady = true;
      };
      document.head.append(script);
    }
  }
};

const startCheckout = async (priceId, btn) => {
  if (!paddleConfigured) {
    showToast('Billing is not configured. Please set up Paddle credentials.', 'error');
  } else if (!paddleReady) {
    showToast('Paddle is still loading. Please try again.', 'warning');
  } else {
    btn.disabled = true;
    btn.textContent = 'Loading...';
    try {
      const data = await apiFetch('/billing/checkout', {
        method: 'POST',
        body: JSON.stringify({ price_id: priceId, success_url: window.location.origin + '/admin/billing' }),
      });
      Paddle.Checkout.open({ transactionId: data.transaction_id });
    } catch (err) {
      showToast('Failed to start checkout: ' + err.message, 'error');
    }
    btn.disabled = false;
    btn.textContent = 'Subscribe';
  }
};

const cancelSubscription = async (btn) => {
  showConfirmDialog(
    'Cancel Subscription?',
    'Are you sure? It will remain active until the end of your current billing period.',
    'Cancel Subscription',
    async () => {
      btn.disabled = true;
      btn.textContent = 'Cancelling...';
      try {
        await apiFetch('/billing/cancel', { method: 'POST' });
        showToast('Subscription cancelled. It will remain active until the end of your billing period.', 'success');
        setTimeout(() => window.location.reload(), 1500);
      } catch (err) {
        btn.disabled = false;
        btn.textContent = 'Cancel Subscription';
        showToast('Failed to cancel: ' + err.message, 'error');
      }
    },
  );
};

const openPortal = async (btn) => {
  btn.disabled = true;
  btn.textContent = 'Loading...';
  try {
    const data = await apiGet('/billing/portal');
    window.open(data.url, '_blank');
  } catch (err) {
    showToast('Failed to open portal: ' + err.message, 'error');
  }
  btn.disabled = false;
  btn.textContent = 'Manage Subscription';
};

loadPaddle();

for (const btn of document.querySelectorAll('.subscribe-btn')) {
  btn.addEventListener('click', () => startCheckout(btn.getAttribute('data-price-id'), btn));
}

document.getElementById('cancel-subscription-btn')?.addEventListener('click', function () {
  cancelSubscription(this);
});

document.getElementById('manage-subscription-btn')?.addEventListener('click', function () {
  openPortal(this);
});
