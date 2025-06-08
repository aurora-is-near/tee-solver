import { ENV } from './env';
import { Config } from './types';

export async function getConfig(): Promise<Config> {
  const module = await import(`./${ENV}`);
  return module.default;
}
