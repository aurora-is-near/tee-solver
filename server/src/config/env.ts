export type Env = 'mainnet' | 'testnet';

export const ENV = requiredEnv('ENV') as Env;

export function requiredEnv(name: string): string {
  return (
    optionalEnv(name) ||
    console.error('Error missing:', name) ||
    process.exit(1)
  );
}

export function optionalEnv(name: string): string | undefined {
  return process.env[name];
}
