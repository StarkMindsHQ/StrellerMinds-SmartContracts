export interface ContractField {
  name: string;
  type: string;
  required: boolean;
}

export interface ContractEndpoint {
  method: string;
  path: string;
  requestFields?: ContractField[];
  responseFields: ContractField[];
  statusCode: number;
}

export interface ContractSchema {
  service: string;
  version: string;
  endpoints: ContractEndpoint[];
}

export interface CompatibilityReport {
  compatible: boolean;
  breakingChanges: BreakingChange[];
  nonBreakingChanges: string[];
  summary: string;
}

export interface BreakingChange {
  type:
    | 'FIELD_REMOVED'
    | 'FIELD_TYPE_CHANGED'
    | 'FIELD_REQUIRED_ADDED'
    | 'STATUS_CODE_CHANGED'
    | 'ENDPOINT_REMOVED';
  location: string;
  description: string;
  oldValue?: string;
  newValue?: string;
}

export function checkCompatibility(
  oldContract: ContractSchema,
  newContract: ContractSchema,
): CompatibilityReport {
  const breakingChanges: BreakingChange[] = [];
  const nonBreakingChanges: string[] = [];

  const oldEndpoints = new Map(
    oldContract.endpoints.map((e) => [`${e.method}:${e.path}`, e]),
  );
  const newEndpoints = new Map(
    newContract.endpoints.map((e) => [`${e.method}:${e.path}`, e]),
  );

  // Removed endpoints
  for (const [key] of oldEndpoints) {
    if (!newEndpoints.has(key)) {
      breakingChanges.push({
        type: 'ENDPOINT_REMOVED',
        location: key,
        description: `Endpoint ${key} was removed`,
        oldValue: key,
      });
    }
  }

  // New endpoints
  for (const [key] of newEndpoints) {
    if (!oldEndpoints.has(key)) {
      nonBreakingChanges.push(`New endpoint added: ${key}`);
    }
  }

  // Changed endpoints
  for (const [key, oldEndpoint] of oldEndpoints) {
    const newEndpoint = newEndpoints.get(key);
    if (!newEndpoint) continue;

    if (oldEndpoint.statusCode !== newEndpoint.statusCode) {
      breakingChanges.push({
        type: 'STATUS_CODE_CHANGED',
        location: key,
        description: `Status code changed`,
        oldValue: String(oldEndpoint.statusCode),
        newValue: String(newEndpoint.statusCode),
      });
    }

    const oldFields = new Map(
      (oldEndpoint.responseFields || []).map((f) => [f.name, f]),
    );
    const newFields = new Map(
      (newEndpoint.responseFields || []).map((f) => [f.name, f]),
    );

    for (const [fieldName, oldField] of oldFields) {
      const newField = newFields.get(fieldName);

      if (!newField) {
        if (oldField.required) {
          breakingChanges.push({
            type: 'FIELD_REMOVED',
            location: `${key} → response.${fieldName}`,
            description: `Required response field "${fieldName}" was removed`,
            oldValue: `${oldField.type} (required)`,
          });
        } else {
          nonBreakingChanges.push(`Optional field removed: ${key}.${fieldName}`);
        }
        continue;
      }

      if (oldField.type !== newField.type) {
        breakingChanges.push({
          type: 'FIELD_TYPE_CHANGED',
          location: `${key} → response.${fieldName}`,
          description: `Type of "${fieldName}" changed`,
          oldValue: oldField.type,
          newValue: newField.type,
        });
      }

      if (!oldField.required && newField.required) {
        breakingChanges.push({
          type: 'FIELD_REQUIRED_ADDED',
          location: `${key} → response.${fieldName}`,
          description: `"${fieldName}" became required`,
          oldValue: 'optional',
          newValue: 'required',
        });
      }
    }

    for (const [fieldName] of newFields) {
      if (!oldFields.has(fieldName)) {
        nonBreakingChanges.push(
          `New response field added: ${key}.response.${fieldName}`,
        );
      }
    }
  }

  const compatible = breakingChanges.length === 0;

  return {
    compatible,
    breakingChanges,
    nonBreakingChanges,
    summary: compatible
      ? `✅ Compatible — ${nonBreakingChanges.length} non-breaking changes`
      : `❌ INCOMPATIBLE — ${breakingChanges.length} breaking change(s) detected`,
  };
}

export function validateVersionCompatibility(
  providerVersion: string,
  consumerVersion: string,
  supportedVersions: string[],
): boolean {
  return (
    supportedVersions.includes(consumerVersion) ||
    providerVersion === consumerVersion
  );
  }
