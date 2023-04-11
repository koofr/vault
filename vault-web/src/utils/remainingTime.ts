import { RemainingTime } from '../vault-wasm/vault-wasm';

export function remainingTimeDisplay(remainingTime: RemainingTime): string {
  let remaining = '';

  if (remainingTime.days > 0) {
    remaining += `${remainingTime.days}d `;
  }

  if (remainingTime.hours > 0) {
    remaining += `${remainingTime.hours}h `;
  }

  if (remainingTime.minutes > 0) {
    remaining += `${remainingTime.minutes}m `;
  }

  remaining += `${remainingTime.seconds | 0}s`;

  return remaining;
}
