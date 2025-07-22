/* This file is generated and managed by tsync */

export interface AccountRecord {
  id: string;
  codeRoot: string;
  storageRoot: string;
  vaultRoot: string;
  nonce: string;
  committed: boolean;
  accountSeed?: Array<number>;
  commitment: string;
}
