
import { test, expect } from '@playwright/test';
import { OnboardingPage, ChatListPage, ChatPage } from './fixtures';

test.describe('Messaging Stability & Presence', () => {
    test('should maintain connection after sending message and update presence', async ({ browser }) => {
        // Setup two isolated browser contexts
        const contextA = await browser.newContext();
        const contextB = await browser.newContext();

        const pageA = await contextA.newPage();
        const pageB = await contextB.newPage();

        // Helpers
        const onboardingA = new OnboardingPage(pageA);
        const onboardingB = new OnboardingPage(pageB);
        const chatListA = new ChatListPage(pageA);
        const chatPageA = new ChatPage(pageA);
        const chatPageB = new ChatPage(pageB);

        // Mock Tauri invoke
        const mockTauri = () => {
            // Mock state
            const mockUser = { id: "user_" + Math.random(), name: "Test User", phone: "1234567890", is_online: true };

            // @ts-ignore
            window.__TAURI__ = {
                core: {
                    invoke: async (cmd: string, args: any) => {
                        console.log(`[Mock] invoke: ${cmd}`, JSON.stringify(args));

                        if (cmd === "get_network_status") return { connected: true, type: "wifi" };
                        if (cmd === "get_ws_auth_token") return "mock-token";

                        // User Service
                        if (cmd === "set_phone_number") {
                            mockUser.phone = args.phone;
                            mockUser.id = args.phone; // Use phone as ID for simplicity
                            return { ...mockUser };
                        }
                        if (cmd === "get_current_user") return undefined; // Initially null? Or mocked user?
                        if (cmd === "update_user") return true;
                        if (cmd === "get_user") return { ...mockUser, id: args.userId };

                        // Contacts
                        if (cmd === "get_contacts") return [];
                        if (cmd === "add_contact") return {
                            id: args.id,
                            name: args.name || "Contact",
                            phone: args.id,
                            is_online: true,
                            avatar_url: null
                        };

                        // Chat Service
                        if (cmd === "create_chat") return {
                            id: "mock_" + args.partner_id,
                            chat_type: "direct",
                            name: args.partner_id,
                            participant: { id: args.partner_id, name: "User B", is_online: true },
                            last_message: null,
                            unread_count: 0
                        };
                        if (cmd === "get_chats") return [];
                        if (cmd === "get_chat_history") return [];

                        if (cmd === "broadcast_message") return true;
                        if (cmd === "broadcast_presence") return true;
                        if (cmd === "disconnect_websocket") return true;

                        return null;
                    },
                },
            };
            // v1 compat just in case
            // @ts-ignore
            window.__TAURI__.invoke = window.__TAURI__.core.invoke;
        };

        await pageA.addInitScript(mockTauri);
        await pageB.addInitScript(mockTauri);

        // Navigate
        await pageA.goto("/?test=onboarding");
        await pageB.goto("/?test=onboarding");

        // 1. Onboard both users
        const userA = await onboardingA.completeOnboarding();
        const userB = await onboardingB.completeOnboarding();

        console.log(`User A: ${userA.phoneNumber}, User B: ${userB.phoneNumber}`);

        // Wait for Chats header
        await expect(pageA.locator('text="Chats"')).toBeVisible();

        // 2. User A opens New Chat Modal
        // Sidebar has a button with tooltip "New chat"
        await pageA.locator('button').filter({ hasText: 'New chat' }).click();

        // 3. Add User B as a contact
        await expect(pageA.locator('text="New chat"')).toBeVisible();
        await pageA.click('button:has-text("Add new contact")');

        await pageA.fill('input[placeholder*="Contact ID"]', userB.phoneNumber);
        await pageA.fill('input[placeholder="Contact Name"]', 'User B');
        await pageA.click('button:has-text("Add Contact")');

        // 4. Start Chat
        // Wait for contact to appear in list and click it
        await pageA.click(`text="User B"`);

        // 5. Monitor Console Logs for Disconnects on Page A
        const disconnectLogs: string[] = [];
        pageA.on('console', msg => {
            if (msg.text().includes('Initiating graceful disconnect')) {
                disconnectLogs.push(msg.text());
                console.error(`[FAILURE] Disconnect detected: ${msg.text()}`);
            }
        });

        // 6. Send Message
        await chatPageA.sendMessage("Hello World");
        await expect(pageA.locator('text="Hello World"')).toBeVisible();

        // 7. Verify NO Disconnects occurred
        // Allow a small grace period for any potential async disconnects
        await pageA.waitForTimeout(1000);
        expect(disconnectLogs.length).toBe(0);

        // 8. Verify Message Delivery to User B (implicitly verifies connection)
        // User B needs to check their specific chat or chat list
        await expect(pageB.locator('text="Hello World"')).toBeVisible();

        // 9. Verify Presence
        // User B disconnects (closes page)
        await pageB.close();

        // User A should see "Last seen" or offline status
        // Depending on UI implementation, it might show "Last seen just now" or just remove the online indicator.
        // Let's assume the indicator is removed or text changes.
        // Investigated UI: ChatPage Header usually shows status.
        // Let's check for "Last seen" text presence.
        await expect(pageA.locator('text="Last seen"')).toBeVisible();
    });
});
