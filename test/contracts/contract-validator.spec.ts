import { checkCompatibility, ContractSchema } from '../../src/shared/contract-validator';

const baseSchema: ContractSchema = {
  service: 'UserService',
  version: '1.0.0',
  endpoints: [
    {
      method: 'GET',
      path: '/users/:id',
      statusCode: 200,
      responseFields: [
        { name: 'id', type: 'string', required: true },
        { name: 'email', type: 'string', required: true },
        { name: 'username', type: 'string', required: true },
        { name: 'createdAt', type: 'string', required: true },
        { name: 'roles', type: 'array', required: true },
      ],
    },
  ],
};

describe('checkCompatibility', () => {
  it('compatible when schemas are identical', () => {
    const report = checkCompatibility(baseSchema, { ...baseSchema });
    expect(report.compatible).toBe(true);
  });

  it('detects removed required field as BREAKING', () => {
    const newSchema = {
      ...baseSchema,
      endpoints: [{
        ...baseSchema.endpoints[0],
        responseFields: baseSchema.endpoints[0].responseFields.filter(f => f.name !== 'email'),
      }],
    };
    const report = checkCompatibility(baseSchema, newSchema);
    expect(report.compatible).toBe(false);
    expect(report.breakingChanges[0].type).toBe('FIELD_REMOVED');
  });

  it('detects type change as BREAKING', () => {
    const newSchema = {
      ...baseSchema,
      endpoints: [{
        ...baseSchema.endpoints[0],
        responseFields: [
          { name: 'id', type: 'number', required: true }, // changed!
          ...baseSchema.endpoints[0].responseFields.slice(1),
        ],
      }],
    };
    const report = checkCompatibility(baseSchema, newSchema);
    expect(report.compatible).toBe(false);
    expect(report.breakingChanges[0].type).toBe('FIELD_TYPE_CHANGED');
  });

  it('detects status code change as BREAKING', () => {
    const newSchema = {
      ...baseSchema,
      endpoints: [{ ...baseSchema.endpoints[0], statusCode: 204 }],
    };
    const report = checkCompatibility(baseSchema, newSchema);
    expect(report.compatible).toBe(false);
    expect(report.breakingChanges[0].type).toBe('STATUS_CODE_CHANGED');
  });

  it('allows adding new field (non-breaking)', () => {
    const newSchema = {
      ...baseSchema,
      endpoints: [{
        ...baseSchema.endpoints[0],
        responseFields: [
          ...baseSchema.endpoints[0].responseFields,
          { name: 'updatedAt', type: 'string', required: false },
        ],
      }],
    };
    const report = checkCompatibility(baseSchema, newSchema);
    expect(report.compatible).toBe(true);
  });
});


