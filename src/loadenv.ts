// loadenv.ts must be imported before anything else in the application!
import process from "node:process";

try {
  process.loadEnvFile(".env");
} catch (e) {
  // If .env is missing, we silently continue because the environment variables
  // might have been injected directly via the shell or deployment platform.
}

export function getReqEnvVariable(name: string): string {
  const value = process.env[name]?.trim();
  if (!value) {
    throw new Error(`Environment variable ${name} is required.`);
  }
  return value;
}

export function getOptionalEnvVariable(name: string): string | undefined {
  return process.env[name];
}
