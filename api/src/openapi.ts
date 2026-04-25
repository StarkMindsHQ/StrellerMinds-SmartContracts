/** OpenAPI 3.0 specification for the Certificate Verification API */
export const openApiSpec = {
  openapi: "3.0.3",
  info: {
    title: "StrellerMinds Certificate Verification API",
    version: "1.0.0",
    description:
      "Public REST API for verifying educational certificates issued on the Stellar blockchain via the StrellerMinds smart contract platform.",
    contact: {
      name: "StrellerMinds",
      url: "https://strellerminds.com",
    },
    license: { name: "MIT" },
  },
  servers: [
    { url: "/api/v1", description: "Current version" },
  ],
  tags: [
    { name: "Auth", description: "Authentication endpoints" },
    { name: "Certificates", description: "Certificate verification and retrieval" },
    { name: "Students", description: "Student certificate listings" },
    { name: "Analytics", description: "Aggregate platform analytics" },
    { name: "Health", description: "Service health and monitoring" },
  ],
  components: {
    securitySchemes: {
      BearerAuth: {
        type: "http",
        scheme: "bearer",
        bearerFormat: "JWT",
        description: "JWT obtained from POST /api/v1/auth/token",
      },
    },
    schemas: {
      ApiResponse: {
        type: "object",
        properties: {
          success: { type: "boolean" },
          data: { nullable: true },
          error: { nullable: true, $ref: "#/components/schemas/ApiError" },
          meta: { $ref: "#/components/schemas/ResponseMeta" },
        },
      },
      ApiError: {
        type: "object",
        properties: {
          code: { type: "string", example: "CERTIFICATE_NOT_FOUND" },
          message: { type: "string" },
          details: { nullable: true },
        },
      },
      ResponseMeta: {
        type: "object",
        properties: {
          requestId: { type: "string", format: "uuid" },
          timestamp: { type: "string", format: "date-time" },
          version: { type: "string", example: "1.0.0" },
        },
      },
      Certificate: {
        type: "object",
        properties: {
          certificateId: { type: "string", description: "64-char hex ID" },
          courseId: { type: "string" },
          student: { type: "string", description: "Stellar address" },
          title: { type: "string" },
          description: { type: "string" },
          metadataUri: { type: "string" },
          issuedAt: { type: "integer", description: "Unix timestamp" },
          expiryDate: { type: "integer", description: "Unix timestamp, 0 = no expiry" },
          status: {
            type: "string",
            enum: ["Active", "Revoked", "Expired", "Suspended", "Reissued"],
          },
          issuer: { type: "string", description: "Stellar address" },
          version: { type: "integer" },
          blockchainAnchor: { type: "string", nullable: true },
          templateId: { type: "string", nullable: true },
          shareCount: { type: "integer" },
        },
      },
      VerificationResult: {
        type: "object",
        properties: {
          certificateId: { type: "string" },
          isValid: { type: "boolean" },
          status: {
            type: "string",
            enum: ["Active", "Revoked", "Expired", "Suspended", "Reissued"],
          },
          verifiedAt: { type: "integer" },
          certificate: {
            nullable: true,
            $ref: "#/components/schemas/Certificate",
          },
          revocationRecord: {
            nullable: true,
            $ref: "#/components/schemas/RevocationRecord",
          },
          message: { type: "string" },
        },
      },
      RevocationRecord: {
        type: "object",
        properties: {
          certificateId: { type: "string" },
          revokedBy: { type: "string" },
          revokedAt: { type: "integer" },
          reason: { type: "string" },
          reissuanceEligible: { type: "boolean" },
        },
      },
      CertificateAnalytics: {
        type: "object",
        properties: {
          totalIssued: { type: "integer" },
          totalRevoked: { type: "integer" },
          totalExpired: { type: "integer" },
          totalReissued: { type: "integer" },
          totalShared: { type: "integer" },
          totalVerified: { type: "integer" },
          activeCertificates: { type: "integer" },
          pendingRequests: { type: "integer" },
          avgApprovalTime: { type: "integer", description: "Seconds" },
          lastUpdated: { type: "integer" },
        },
      },
    },
  },
  paths: {
    "/auth/token": {
      post: {
        tags: ["Auth"],
        summary: "Issue a JWT access token",
        requestBody: {
          required: true,
          content: {
            "application/json": {
              schema: {
                type: "object",
                required: ["apiKey"],
                properties: {
                  apiKey: { type: "string", minLength: 16 },
                },
              },
            },
          },
        },
        responses: {
          "200": {
            description: "Token issued",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: {
                          type: "object",
                          properties: {
                            accessToken: { type: "string" },
                            tokenType: { type: "string", example: "Bearer" },
                            expiresIn: { type: "string", example: "1h" },
                            scope: { type: "array", items: { type: "string" } },
                          },
                        },
                      },
                    },
                  ],
                },
              },
            },
          },
          "401": { description: "Invalid API key" },
          "429": { description: "Rate limit exceeded" },
        },
      },
    },
    "/certificates/{id}/verify": {
      get: {
        tags: ["Certificates"],
        summary: "Verify a certificate (public)",
        description:
          "Checks whether a certificate is valid, active, and not expired. No authentication required. Rate-limited to 100 req/min.",
        parameters: [
          {
            name: "id",
            in: "path",
            required: true,
            description: "64-character hex certificate ID",
            schema: { type: "string", pattern: "^[0-9a-fA-F]{64}$" },
          },
        ],
        responses: {
          "200": {
            description: "Verification result",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: { $ref: "#/components/schemas/VerificationResult" },
                      },
                    },
                  ],
                },
              },
            },
          },
          "400": { description: "Invalid certificate ID format" },
          "429": { description: "Rate limit exceeded" },
          "502": { description: "Blockchain query failed" },
        },
      },
    },
    "/certificates/{id}": {
      get: {
        tags: ["Certificates"],
        summary: "Get certificate details",
        security: [{ BearerAuth: [] }],
        parameters: [
          {
            name: "id",
            in: "path",
            required: true,
            schema: { type: "string" },
          },
        ],
        responses: {
          "200": {
            description: "Certificate data",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: { $ref: "#/components/schemas/Certificate" },
                      },
                    },
                  ],
                },
              },
            },
          },
          "401": { description: "Unauthorized" },
          "404": { description: "Certificate not found" },
        },
      },
    },
    "/certificates/{id}/revocation": {
      get: {
        tags: ["Certificates"],
        summary: "Get revocation record",
        security: [{ BearerAuth: [] }],
        parameters: [
          {
            name: "id",
            in: "path",
            required: true,
            schema: { type: "string" },
          },
        ],
        responses: {
          "200": {
            description: "Revocation record",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: { $ref: "#/components/schemas/RevocationRecord" },
                      },
                    },
                  ],
                },
              },
            },
          },
          "401": { description: "Unauthorized" },
          "404": { description: "No revocation record found" },
        },
      },
    },
    "/students/{address}/certificates": {
      get: {
        tags: ["Students"],
        summary: "List student certificates",
        security: [{ BearerAuth: [] }],
        parameters: [
          {
            name: "address",
            in: "path",
            required: true,
            description: "Stellar public key (G...)",
            schema: { type: "string" },
          },
        ],
        responses: {
          "200": {
            description: "List of certificate IDs",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: {
                          type: "object",
                          properties: {
                            student: { type: "string" },
                            certificateIds: {
                              type: "array",
                              items: { type: "string" },
                            },
                            total: { type: "integer" },
                          },
                        },
                      },
                    },
                  ],
                },
              },
            },
          },
          "400": { description: "Invalid Stellar address" },
          "401": { description: "Unauthorized" },
        },
      },
    },
    "/analytics": {
      get: {
        tags: ["Analytics"],
        summary: "Get aggregate certificate analytics",
        security: [{ BearerAuth: [] }],
        responses: {
          "200": {
            description: "Analytics data",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: { $ref: "#/components/schemas/CertificateAnalytics" },
                      },
                    },
                  ],
                },
              },
            },
          },
          "401": { description: "Unauthorized" },
        },
      },
    },
  },
};
