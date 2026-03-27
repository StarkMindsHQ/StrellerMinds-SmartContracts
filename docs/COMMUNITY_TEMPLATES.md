# Community Engagement Templates

This document provides templates for common community engagement scenarios. Use these templates to ensure consistent, welcoming, and helpful communication.

## Table of Contents

1. [Welcome Messages](#welcome-messages)
2. [Issue Response Templates](#issue-response-templates)
3. [Pull Request Templates](#pull-request-templates)
4. [Recognition Templates](#recognition-templates)
5. [Event Communication](#event-communication)
6. [Difficult Situations](#difficult-situations)
7. [Offboarding Templates](#offboarding-templates)

---

## Welcome Messages

### New Contributor Welcome

```markdown
🎉 **Welcome to StrellerMinds-SmartContracts!** @username

Hi there! We're thrilled to have you join our community! 👋

Thank you for your interest in contributing. Here are some resources to get you started:

📚 **Getting Started:**
- [README.md](../README.md) - Project overview
- [CONTRIBUTING.md](docs/contributing.md) - Contribution guide  
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) - Community guidelines

🚀 **First Steps:**
1. Browse our [good first issues](link-to-issues)
2. Join our [GitHub Discussions](link-to-discussions)
3. Check out our [documentation](docs/)

💡 **Need Help?**
- Ask questions in GitHub Discussions
- Attend weekly office hours (schedule in discussions)
- Tag maintainers for assistance

We're here to support you every step of the way. Don't hesitate to ask questions - there are no silly questions in this community!

Looking forward to your contributions! 🌟

Best regards,
The StrellerMinds Team
```

### First Issue Assignment

```markdown
Great to see your interest in contributing! 🎯

**Issue #{{issue_number}}: {{issue_title}}** has been assigned to you.

📋 **Next Steps:**
1. Comment on the issue to confirm you're working on it
2. Fork the repository and create a branch
3. Implement your solution
4. Write tests for your changes
5. Submit a pull request

📖 **Resources:**
- Development setup: [Setup Guide](docs/development.md)
- Testing: [Testing Guide](docs/testing.md)
- Code style: [Style Guide](docs/CODE_STYLE.md)

⏰ **Timeline:**
Please provide an update within 2 weeks, even if it's just to say you're still working on it. If you need more time or run into blockers, just let us know!

🆘 **Stuck?**
Don't struggle in silence! Ask for help in:
- The issue comments
- GitHub Discussions
- Office hours

We're excited to see your contribution! 💪
```

### First PR Celebration

```markdown
🎊 **Congratulations on Your First PR!** 🎊

@username This is amazing! Thank you for your contribution to StrellerMinds-SmartContracts! 

✨ **What happens next:**
1. Automated CI checks will run
2. A maintainer will review your code (typically within 5-7 days)
3. You may receive feedback or suggestions
4. Once approved, we'll merge your PR!

📝 **Tips:**
- Respond to any feedback promptly
- Don't worry if revisions are requested - it's all part of the process
- Ask questions if anything is unclear

🌱 **This is just the beginning!**
After this merges, you'll be eligible to take on more complex issues. We'd love to see you become a regular contributor!

Celebrating your first step into our community! 🥳

Warmly,
Your StrellerMinds Maintainers
```

---

## Issue Response Templates

### Bug Report Acknowledgment

```markdown
Thank you for reporting this bug! 🐛

**Issue #{{issue_number}}** has been received and labeled for triage.

🔍 **What we're doing:**
- A maintainer will review this within 24-48 hours
- We'll attempt to reproduce the issue
- Security implications will be assessed

📋 **In the meantime:**
- Please monitor this issue for follow-up questions
- If you discover additional details, add them as comments
- For urgent security issues, contact maintainers directly

🏷️ **Current Status:** `status: triage`

We appreciate you taking the time to report this thoroughly. Clear bug reports like yours help us improve the project significantly!

Best,
The Maintenance Team
```

### Feature Request Response

```markdown
Thanks for this thoughtful feature request! 💡

**Feature: {{feature_title}}**

I can see how this would {{benefit_description}}. Great thinking!

📊 **Review Process:**
1. Technical feasibility assessment (3-5 days)
2. Alignment with project roadmap evaluation
3. Community input gathering (if needed)
4. Decision and timeline communication

🗳️ **Community Input Welcome**
Others: What do you think about this feature? Please share your perspectives below!

📅 **Next Steps:**
We'll update this issue within a week with our decision. If accepted, we'll discuss implementation approaches and timeline.

Whether or not we move forward, thank you for helping us think about how to make the project better! 🙏

Cheers,
Maintainers
```

### Question Response (Answer Known)

```markdown
Great question! 🤔

**Answer:**
{{Provide clear, concise answer with example if applicable}}

**Example:**
```rust
// Code example if relevant
```

**Additional Resources:**
- [Link to documentation]
- [Related issue/discussion]

Does this answer your question? Feel free to ask follow-ups! 😊

Helpful!
```

### Question Response (Redirect to Documentation)

```markdown
Good question! 📚

This is covered in our documentation here: [{{doc_title}}]({{doc_link}})

**Quick Summary:**
{{Brief 1-2 sentence summary}}

**Key Points:**
- Point 1
- Point 2
- Point 3

If the documentation isn't clear or you have follow-up questions, please don't hesitate to ask! We're always working to improve our docs based on questions like this.

Was this helpful? Let us know! 👍
```

### Cannot Reproduce Bug

```markdown
Thank you for your patience while we investigated this bug. 🔍

We've attempted to reproduce the issue using the steps provided, but haven't been able to replicate it in our environment.

**What we tried:**
- Environment: {{details}}
- Steps: {{what was attempted}}
- Result: {{outcome}}

❓ **Could you help us with:**
1. Confirming your exact environment versions?
2. Trying on latest main branch?
3. Providing additional logs or details?

It's possible this is:
- Environment-specific
- Already fixed in newer versions
- Related to specific configuration

Let's work together to get to the bottom of this! 🕵️

Thanks,
Investigation Team
```

### Closing Issue (Completed)

```markdown
✅ **Issue Resolved!**

Thanks to {{contributor_names}} for the excellent work on this!

**What was done:**
{{Brief summary of fix}}

**PR:** #{{pr_number}}

🎉 This fix will be included in the next release (v{{version}}).

If anyone experiences related issues or has questions about this fix, feel free to comment here or open a new issue.

Marking as complete. Thanks everyone! 🙌
```

### Declining Feature Request

```markdown
Thank you for this feature suggestion! 💭

After careful consideration, we've decided not to move forward with this at this time.

**Reasons:**
{{Honest but tactful explanation}}
- e.g., "This falls outside our current scope"
- e.g., "We're focusing on core functionality first"
- e.g., "Similar functionality exists via alternative approach"

**Alternatives:**
{{Suggest workarounds if available}}
- e.g., "You could implement this as an extension"
- e.g., "Consider using X pattern instead"

**Future Possibility:**
We may revisit this in the future as the project evolves. We'll keep this issue open for reference and community input.

We truly appreciate you sharing your ideas! Even when we can't implement everything suggested, these discussions help us understand community needs better. 🙏

Would you be interested in exploring other ways to contribute? We'd love your input on {{related_areas}}!

Best regards,
Maintainers
```

---

## Pull Request Templates

### PR Review Request to Author

```markdown
Hi @{{author}}! 👋

Thanks for this PR! I've reviewed your changes and have some feedback.

**Overall Impression:**
{{Positive opening - find something genuine to praise}}

**Suggestions:**

🔧 **Technical:**
1. {{Specific technical feedback}}
2. {{Another point if needed}}

📝 **Documentation:**
- {{Doc-related feedback}}

🎨 **Style:**
- {{Style suggestions}}

**Questions:**
- {{Any clarifying questions}}

**Next Steps:**
Please address the above when you get a chance. No rush - take your time to thoughtfully respond.

Let me know if any feedback is unclear or if you'd like to discuss alternatives! Happy to hop on a call if that's easier.

Looking forward to your revisions! 🚀

Cheers,
{{reviewer_name}}
```

### Approving PR Review

```markdown
✅ **Looks Great!**

Nice work, @{{author}}! This is exactly what we needed.

**What I like:**
- {{Specific positive points}}
- {{Clean code / good tests / etc.}}

**Approval:** ✅ Approved

I'll merge this once CI passes. Expect it to be live in the next release!

🎉 Excellent contribution - thank you!

Best,
{{reviewer_name}}
```

### Requesting PR Changes

```markdown
Thanks for your work on this, @{{author}}! 🙏

Before we can merge, we need some changes:

**Required Changes:**
🔴 **Must Fix:**
1. {{Critical issue 1}}
2. {{Critical issue 2}}

**Suggested Improvements:**
🟡 **Nice to Have:**
1. {{Enhancement suggestion}}
2. {{Another improvement}}

**Questions to Address:**
❓ {{Question 1}}
❓ {{Question 2}}

**Timeline:**
Could you address these within the next week? If you need more time or have questions, just let me know!

**Need Help?**
If anything is unclear or you'd like to pair on this, I'm happy to help. Just say the word!

Looking forward to your updates! 💪

Best,
{{reviewer_name}}
```

### Merging PR Announcement

```markdown
🎉 **Merged!** 

Your PR has been merged to main! 

**Contributor:** @{{author}}  
**PR:** #{{pr_number}}: {{title}}  
**Release:** Included in v{{version}}

**What this adds:**
{{Brief summary of contribution}}

🌟 **Thank you!**
This is a valuable addition to the project. Your contribution makes StrellerMinds-SmartContracts better for everyone!

**What's Next?**
- Check out other open issues if you're looking for more ways to contribute
- Help review others' PRs
- Share your experience in discussions

Congratulations again! 🥳

Cheers,
Merge Masters
```

---

## Recognition Templates

### Contributor Spotlight

```markdown
🌟 **Contributor Spotlight: @{{username}}** 🌟

This month, we're celebrating {{contributor_name}} for their outstanding contributions!

**Contributions:**
- 📝 {{Number}} PRs merged
- 💬 {{Number}} helpful issue comments
- 🤝 {{Number}} newcomers helped
- 📚 {{Specific notable contribution}}

**About {{name}}:**
{{Brief bio if provided, or description of their journey}}

**Words from {{name}}:**
> "{{Quote from contributor about their experience}}"

**Impact:**
Thanks to {{name}}, we now have {{specific improvement}}. This benefits {{who benefits}} by {{how it helps}}.

**Join us in thanking {{name}}!** 👏👏👏

Want to contribute like {{name}}? Start with our [good first issues](link)!

#CommunitySpotlight #OpenSource #Blockchain
```

### Milestone Celebration

```markdown
🎊 **Milestone Alert: {{milestone}}!** 🎊

We've reached an amazing milestone thanks to contributors like you!

**The Number:** {{milestone_metric}}  
**Achieved:** {{date}}  
**Journey:** Started {{start_date}}

**Highlights Along the Way:**
- Major releases: {{list}}
- Key features added: {{summary}}
- Community growth: {{stats}}

**Top Contributors This Period:**
{{List top contributors with specific contributions}}

**What This Means:**
{{Context about why this milestone matters}}

**What's Next:**
{{Upcoming goals and initiatives}}

**Thank You!**
Every contributor, from first-timers to veterans, helped make this possible. You're all amazing! 🙌

Here's to the next milestone! 🚀

#Milestone #Community #Achievement
```

### Badge Award Template

```markdown
🏅 **Badge Award: {{badge_name}}** 🏅

Presented to: @{{username}}  
Date: {{date}}  
Category: {{badge_category}}

**Badge Description:**
{{What the badge represents}}

**Why They Earned It:**
{{Specific achievements that earned this badge}}

**Impact:**
{{How their contributions made a difference}}

**Badge Level:** {{level if applicable}}

Congratulations, {{username}}! Your contributions inspire us all! 🌟

Show your support with a congratulatory message below! 👇

#Badges #Recognition #Community
```

---

## Event Communication

### Event Announcement

```markdown
📅 **Event Announcement: {{event_name}}** 📅

**Save the Date!**

🗓️ **When:** {{date and time}}  
📍 **Where:** {{location or online link}}  
🎯 **Topic:** {{event focus}}

**What to Expect:**
{{Description of event format and content}}

**Who Should Attend:**
{{Target audience}}

**RSVP:**
{{Registration link and deadline}}

**Speakers/Presenters:**
{{Speaker names and bios if applicable}}

**Share Widely!**
Know someone who'd benefit from this? Please share! 🔄

Questions? Drop them below! 👇

See you there! 🎉

#Event #Community #Learning
```

### Event Reminder

```markdown
⏰ **Reminder: {{event_name}} is {{timeframe}}!** ⏰

Hey everyone! Quick reminder that {{event_name}} is happening {{when}}!

**Details:**
📅 Date: {{date}}  
⏰ Time: {{time}}  
🔗 Link: {{join_link}}

**Still Time to Register!**
Sign up here: {{registration_link}}

**What We'll Cover:**
- Agenda item 1
- Agenda item 2
- Q&A session

**Prepare:**
{{Any preparation attendees should do}}

Can't make it live? Register anyway - we'll send the recording! 📹

See you soon! 👋

#Reminder #Event #DontMissOut
```

### Post-Event Thank You

```markdown
🎉 **Thank You - {{event_name}} Success!** 🎉

Wow! What an amazing event! Thank you to everyone who made {{event_name}} such a success!

**By the Numbers:**
- 👥 {{number}} attendees
- 💬 {{number}} questions asked
- 🌍 {{number}} countries represented
- ⭐ {{rating}} average rating

**Highlights:**
{{Brief summary of key moments}}

**Recording & Materials:**
📹 Watch: {{video_link}}  
📄 Slides: {{slide_link}}  
💻 Code examples: {{repo_link}}

**Special Thanks:**
- Our speakers: {{names}}
- Event organizers: {{names}}
- Sponsors (if applicable): {{names}}
- All attendees who made it great!

**Feedback:**
Help us improve! Take 2 minutes to fill out: {{feedback_survey_link}}

**Stay Tuned:**
More events coming soon! Follow this repo for announcements.

Until next time! 🚀

#EventSuccess #ThankYou #Community
```

---

## Difficult Situations

### Code of Conduct Reminder

```markdown
📋 **Community Guidelines Reminder**

Hi everyone,

I want to take a moment to remind our community about maintaining a respectful and inclusive environment.

**Our Values:**
- Respect for all participants
- Focus on constructive criticism
- Welcoming attitude toward newcomers
- Professional communication

**Recent Concerns:**
{{General description without calling out individuals}}

**Moving Forward:**
Let's all recommit to:
- Listening actively
- Assuming good intentions
- Disagreeing respectfully
- Focusing on ideas, not personalities

**Resources:**
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Community Guidelines](COMMUNITY.md)

**Need Support?**
If you've experienced or witnessed behavior that violates our CoC, please reach out to maintainers privately.

Together, we maintain a healthy community! 💪

Thank you,
Community Team
```

### De-escalation Message

```markdown
🤝 **Let's Pause and Reset**

Hi @{{participants}},

I notice this discussion is getting heated. Let's take a step back.

**Reminder:**
We're all here because we care about this project. Disagreements are natural and often lead to better outcomes when discussed constructively.

**Suggestions:**
1. Take a break from this thread (24-48 hours)
2. Reflect on shared goals
3. Consider alternative perspectives
4. Return with fresh eyes

**Alternative Approaches:**
- Continue via private discussion with mediator
- Schedule a call to talk through differences
- Document both positions for broader community input

**My Role:**
I'm here to facilitate productive discussion. If helpful, I can:
- Moderate continued discussion
- Arrange mediation
- Gather broader community input

**Next Steps:**
Let's reconvene {{timeframe}}. In the meantime, please keep communications respectful.

We value all perspectives here. Let's work through this together. 🤝

Best,
{{facilitator_name}}
```

### Temporary Ban Notification

```markdown
**Subject: Temporary Suspension from Community Participation**

Dear {{username}},

This message is to inform you that your participation in the StrellerMinds-SmartContracts community spaces has been temporarily suspended for {{duration}}.

**Reason:**
{{Clear, factual description of violations}}

**Relevant Code of Conduct Sections:**
{{Specific sections violated}}

**Suspension Details:**
- **Start Date:** {{date}}
- **End Date:** {{date}}
- **Conditions:** During this period, you may not participate in {{list of spaces}}

**Path Forward:**
Before reinstatement, we ask that you:
1. Reflect on the behavior that led to this suspension
2. Review our Code of Conduct
3. Confirm your commitment to following it going forward

**Appeal Process:**
If you wish to appeal this decision, you may do so by {{appeal_process}}.

**Support:**
We want you to succeed in our community. If you need clarification or support, please contact {{contact}}.

We hope to welcome you back after this suspension period.

Regards,
Community Moderation Team
```

---

## Offboarding Templates

### Contributor Departure (Voluntary)

```markdown
👋 **Farewell and Thank You: @{{username}}**

It's with mixed emotions that we share {{contributor_name}}'s departure from active contribution.

**Their Journey:**
- Contributed for: {{duration}}
- Major contributions: {{list}}
- Impact: {{specific improvements}}

**Words from {{name}}:**
> "{{Their farewell message if provided}}"

**Our Wishes:**
While we're sad to see you go, we completely understand and support your decision. Thank you for everything you've given to this community!

**Door Always Open:**
You'll always have a home here if you choose to return. Until then, we wish you all the best in your future endeavors! 🌟

**Legacy:**
Your contributions will continue to benefit users and developers for years to come. That's lasting impact!

With gratitude,
The StrellerMinds Community

P.S. Stay in touch! {{social_links if appropriate}}
```

### Maintainer Transition

```markdown
🔄 **Maintainer Transition Announcement**

Dear Community,

{{maintainer_name}} will be transitioning from their maintainer role effective {{date}}.

**Their Service:**
- Role: {{specific responsibilities}}
- Tenure: {{duration}}
- Key achievements: {{list}}

**Transition Plan:**
{{Details about how responsibilities will be reassigned}}

**Interim Coverage:**
{{Who is covering their responsibilities}}

**Farewell Event:**
{{Details about any farewell gathering if applicable}}

**Thank You:**
We cannot thank {{name}} enough for their dedication, expertise, and leadership. The project is vastly better because of their contributions.

**Stay Connected:**
{{name}} will remain part of our community as {{new_role if applicable}}.

Please join us in expressing gratitude for their service! 🙏

Best regards,
Project Leadership
```

---

## Usage Guidelines

### When to Use Templates

✅ **Appropriate Uses:**
- Standard responses to common situations
- Ensuring consistent communication
- Training new maintainers
- Saving time on routine messages
- Maintaining professional tone

❌ **When to Customize:**
- Sensitive situations requiring personal touch
- Complex technical discussions
- Conflict resolution scenarios
- Unique circumstances
- When relationship allows informal tone

### Customization Tips

**Always Personalize:**
- Add recipient's name
- Reference specific details
- Adjust tone to relationship
- Include your authentic voice

**Keep Flexible:**
- Templates are guides, not scripts
- Adapt to situation
- Add empathy and warmth
- Be genuine, not robotic

### Tone Guidelines

**Default Tone:**
- Friendly and welcoming
- Professional but warm
- Encouraging and supportive
- Clear and direct

**Adjust Based On:**
- Severity of situation
- Relationship with recipient
- Community norms
- Cultural context

---

**Version**: 1.0.0  
**Last Updated**: 2026-03-27  
**Maintained By**: Community Team  
**Usage**: Freely adaptable for project needs
