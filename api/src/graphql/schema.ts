import {
  GraphQLSchema,
  GraphQLObjectType,
  GraphQLString,
  GraphQLInt,
  GraphQLList,
  GraphQLNonNull,
} from "graphql";

// ── Types ─────────────────────────────────────────────────────────────────────

const CertificateType = new GraphQLObjectType({
  name: "Certificate",
  fields: {
    id: { type: new GraphQLNonNull(GraphQLString) },
    studentId: { type: new GraphQLNonNull(GraphQLString) },
    courseId: { type: new GraphQLNonNull(GraphQLString) },
    issuedAt: { type: GraphQLString },
    status: { type: GraphQLString },
  },
});

const StudentType = new GraphQLObjectType({
  name: "Student",
  fields: {
    id: { type: new GraphQLNonNull(GraphQLString) },
    name: { type: GraphQLString },
    email: { type: GraphQLString },
    certificates: {
      type: new GraphQLList(CertificateType),
      resolve: () => [], // placeholder — wire to real data layer as needed
    },
  },
});

const RateLimitInfoType = new GraphQLObjectType({
  name: "RateLimitInfo",
  fields: {
    limit: { type: new GraphQLNonNull(GraphQLInt) },
    remaining: { type: new GraphQLNonNull(GraphQLInt) },
    resetAt: { type: new GraphQLNonNull(GraphQLInt) },
  },
});

// ── Root Query ────────────────────────────────────────────────────────────────

const QueryType = new GraphQLObjectType({
  name: "Query",
  fields: {
    health: {
      type: new GraphQLNonNull(GraphQLString),
      resolve: () => "ok",
    },
    student: {
      type: StudentType,
      args: { id: { type: new GraphQLNonNull(GraphQLString) } },
      resolve: (_root, args: { id: string }) => ({
        id: args.id,
        name: "Placeholder",
        email: "placeholder@example.com",
      }),
    },
    certificates: {
      type: new GraphQLList(CertificateType),
      args: { studentId: { type: new GraphQLNonNull(GraphQLString) } },
      resolve: () => [],
    },
    rateLimitInfo: {
      type: RateLimitInfoType,
      resolve: () => ({ limit: 60, remaining: 60, resetAt: Math.ceil(Date.now() / 1000) + 60 }),
    },
  },
});

export const schema = new GraphQLSchema({ query: QueryType });
