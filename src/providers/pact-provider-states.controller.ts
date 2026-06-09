import { Controller, Post, Body, HttpCode } from '@nestjs/common';

interface ProviderStateRequest {
  state: string;
  action: 'setup' | 'teardown';
  params?: Record<string, unknown>;
}

@Controller('_pact')
export class PactProviderStatesController {
  private readonly stateHandlers: Record
    string,
    { setup: () => Promise<void>; teardown?: () => Promise<void> }
  > = {
    'a user with id user-abc-123 exists': {
      setup: async () => {
        // TODO: await userRepo.save({ id: 'user-abc-123', ... })
      },
      teardown: async () => {
        // TODO: await userRepo.delete('user-abc-123')
      },
    },
    'a user with id nonexistent does not exist': {
      setup: async () => {
        // TODO: ensure user doesn't exist
      },
    },
    'users exist in the system': {
      setup: async () => {
        // TODO: seed test users
      },
    },
    'the user system is ready to accept new users': {
      setup: async () => {
        // TODO: clear conflicting data
      },
    },
  };

  @Post('provider-states')
  @HttpCode(200)
  async handleProviderState(@Body() body: ProviderStateRequest) {
    const { state, action } = body;
    const handler = this.stateHandlers[state];

    if (!handler) {
      console.warn(`[Pact] No handler for state: "${state}"`);
      return { state, result: 'no-op' };
    }

    if (action === 'setup') await handler.setup();
    else if (action === 'teardown' && handler.teardown) await handler.teardown();

    return { state, result: 'ok' };
  }
}
