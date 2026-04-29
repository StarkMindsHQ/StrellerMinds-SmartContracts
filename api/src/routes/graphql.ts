import { Router } from "express";
import { createHandler } from "graphql-http/lib/use/express";
import { graphqlRateLimiter } from "../middleware/graphqlRateLimiter";
import { schema } from "../graphql/schema";

const router = Router();

// Apply GraphQL-specific rate limiting before the handler
router.use(graphqlRateLimiter());

// Mount the GraphQL handler
router.all("/", createHandler({ schema }));

export default router;
