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
    { name: "Rate Limiting", description: "Per-user rate limit status and tier info" },
    { name: "CDN", description: "Cache invalidation and CDN configuration" },
    { name: "Performance", description: "Query optimization, cache, and backend pool diagnostics" },
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
      UserRateLimitStatus: {
        type: "object",
        properties: {
          userId: { type: "string" },
          tier: { type: "string", enum: ["free", "pro", "enterprise", "internal"] },
          consumed: { type: "integer", description: "Requests consumed in current window" },
          remaining: { type: "integer", description: "Requests remaining in current window" },
          limit: { type: "integer", description: "Requests per minute for this tier" },
          burstLimit: { type: "integer", description: "Burst allowance per 10 seconds" },
          burstConsumed: { type: "integer" },
          resetAt: { type: "integer", description: "Unix timestamp when window resets" },
          throttled: { type: "boolean" },
          windowRemainingMs: { type: "integer" },
        },
      },
      QueryOptimizationReport: {
        type: "object",
        properties: {
          timestamp: { type: "string", format: "date-time" },
          targets: {
            type: "object",
            properties: {
              avgQueryTimeMs: { type: "number" },
              loadReductionPercent: { type: "number" },
            },
          },
          summary: {
            type: "object",
            properties: {
              averageQueryTimeMs: { type: "number" },
              meetsAverageQueryTarget: { type: "boolean" },
              estimatedLoadReductionPercent: { type: "number" },
              meetsLoadReductionTarget: { type: "boolean" },
              cacheHitRatio: { type: "number" },
              cacheEntries: { type: "integer" },
              totalRequests: { type: "integer" },
              totalBackendCalls: { type: "integer" },
              totalBackendRequestCalls: { type: "integer" },
              totalBackgroundRefreshes: { type: "integer" },
              slowQueries: { type: "integer" },
            },
          },
          cache: {
            type: "object",
            properties: {
              maxEntries: { type: "integer" },
              activeEntries: { type: "integer" },
              hitRatio: { type: "number" },
            },
          },
          pool: {
            type: "object",
            properties: {
              size: { type: "integer" },
              available: { type: "integer" },
              inFlightBackendQueries: { type: "integer" },
              configuredRpcUrls: { type: "array", items: { type: "string" } },
              roundRobinCursor: { type: "integer" },
            },
          },
          slowQueries: {
            type: "array",
            items: {
              type: "object",
              properties: {
                query: { type: "string" },
                slowCount: { type: "integer" },
                lastDurationMs: { type: "number" },
                averageDurationMs: { type: "number" },
                p95DurationMs: { type: "number" },
              },
            },
          },
          topQueries: {
            type: "array",
            items: {
              type: "object",
              properties: {
                query: { type: "string" },
                requests: { type: "integer" },
                averageDurationMs: { type: "number" },
                p95DurationMs: { type: "number" },
                p99DurationMs: { type: "number" },
                cacheHitRatio: { type: "number" },
                estimatedLoadReductionPercent: { type: "number" },
                backendCalls: { type: "integer" },
                effectiveTtlMs: { type: "integer" },
              },
            },
          },
          optimization: {
            type: "object",
            properties: {
              adaptiveCachingEnabled: { type: "boolean" },
              knownSingletonPrewarmQueries: {
                type: "array",
                items: { type: "string" },
              },
              recommendations: {
                type: "array",
                items: { type: "string" },
              },
              indexOptimizationSpecs: {
                type: "array",
                items: {
                  type: "object",
                  properties: {
                    query: { type: "string" },
                    backend: { type: "string" },
                    accessPattern: { type: "string" },
                    recommendedLookupKey: { type: "string" },
                    recommendedIndexSpec: { type: "string" },
                    cacheCoverage: { type: "string" },
                    directLookupCovered: { type: "boolean" },
                    notes: { type: "string" },
                  },
                },
              },
            },
          },
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
    "/rate-limit/status": {
      get: {
        tags: ["Rate Limiting"],
        summary: "Get current user rate limit status",
        security: [{ BearerAuth: [] }],
        responses: {
          "200": {
            description: "Rate limit status for the authenticated user",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: { $ref: "#/components/schemas/UserRateLimitStatus" },
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
    "/rate-limit/tiers": {
      get: {
        tags: ["Rate Limiting"],
        summary: "Get available rate limit tier definitions (public)",
        responses: {
          "200": {
            description: "Tier definitions",
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
                            tiers: {
                              type: "array",
                              items: {
                                type: "object",
                                properties: {
                                  tier: { type: "string", enum: ["free", "pro", "enterprise", "internal"] },
                                  requestsPerMinute: { type: "integer" },
                                  burstAllowance: { type: "integer" },
                                },
                              },
                            },
                          },
                        },
                      },
                    },
                  ],
                },
              },
            },
          },
        },
      },
    },
    "/performance/query-optimization": {
      get: {
        tags: ["Performance"],
        summary: "Get query optimization diagnostics and recommendations",
        responses: {
          "200": {
            description: "Current query optimization report",
            content: {
              "application/json": {
                schema: {
                  allOf: [
                    { $ref: "#/components/schemas/ApiResponse" },
                    {
                      properties: {
                        data: { $ref: "#/components/schemas/QueryOptimizationReport" },
                      },
                    },
                  ],
                },
              },
            },
          },
        },
      },
    },
    "/performance/query-cache/invalidate": {
      post: {
        tags: ["Performance"],
        summary: "Invalidate query cache entries by query or key prefix",
        security: [{ BearerAuth: [] }],
        requestBody: {
          required: false,
          content: {
            "application/json": {
              schema: {
                type: "object",
                properties: {
                  query: { type: "string" },
                  keyPrefix: { type: "string" },
                },
              },
            },
          },
        },
        responses: {
          "200": { description: "Query cache invalidated" },
          "400": { description: "Invalid request body" },
          "401": { description: "Unauthorized" },
        },
      },
    },
    "/cdn/invalidate": {
      post: {
        tags: ["CDN"],
        summary: "Trigger cache invalidation for path patterns",
        description: "Requires HMAC-SHA256 signature in X-CDN-Signature header (sha256=<hex>).",
        requestBody: {
          required: true,
          content: {
            "application/json": {
              schema: {
                type: "object",
                required: ["patterns"],
                properties: {
                  patterns: { type: "array", items: { type: "string" }, maxItems: 50 },
                  reason: { type: "string" },
                },
              },
            },
          },
        },
        responses: {
          "200": { description: "Invalidation triggered" },
          "401": { description: "Missing or invalid signature" },
        },
      },
    },
    "/cdn/status": {
      get: {
        tags: ["CDN"],
        summary: "CDN configuration and active invalidations",
        responses: {
          "200": { description: "CDN status" },
        },
      },
    },
  },
};
