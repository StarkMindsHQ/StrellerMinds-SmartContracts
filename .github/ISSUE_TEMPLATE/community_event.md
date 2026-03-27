---
name: 🎉 Community Event Proposal
description: Propose a community event, meetup, or workshop
title: "[EVENT] <Event name/location>"
labels: ["community", "event", "proposal"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for organizing a community event! 🎉
        
        We support various types of events:
        - Local meetups
        - Workshops and tutorials
        - Hackathon participation
        - Conference presentations
        - Online webinars
        
  - type: input
    id: event-type
    attributes:
      label: Event Type
      description: What kind of event is this?
      placeholder: e.g., Meetup, Workshop, Conference Talk, Hackathon
    validations:
      required: true
      
  - type: input
    id: location
    attributes:
      label: Location
      description: Where will this take place? (City/Country or Online)
      placeholder: e.g., Berlin, Germany or Online via Zoom
    validations:
      required: true
      
  - type: input
    id: proposed-date
    attributes:
      label: Proposed Date
      description: When do you plan to hold this event?
      placeholder: YYYY-MM-DD or Month Year
    validations:
      required: true
      
  - type: textarea
    id: description
    attributes:
      label: Event Description
      description: Describe the event format, topics, and target audience
      placeholder: What will happen at this event?
    validations:
      required: true
      
  - type: textarea
    id: expected-attendance
    attributes:
      label: Expected Attendance
      description: How many people do you expect?
      placeholder: Estimated number and audience composition
      
  - type: textarea
    id: support-needed
    attributes:
      label: Support Needed from Project
      description: What kind of support do you need?
      placeholder: |
        Examples:
        - Presentation materials
        - Speaker training
        - Swag/merchandise
        - Travel funding
        - Technical setup help
        - Promotion assistance
      
  - type: textarea
    id: organizer-info
    attributes:
      label: Organizer Information
      description: Tell us about yourself and your experience organizing events
      placeholder: Your background and event organizing experience
      
  - type: checkboxes
    id: code-of-conduct
    attributes:
      label: Code of Conduct Agreement
      description: Do you agree to follow and enforce the Code of Conduct at this event?
      options:
        - label: I agree to follow and enforce the Code of Conduct
          required: true
          
  - type: checkboxes
    id: inclusivity
    attributes:
      label: Inclusivity Commitment
      description: Will this event be accessible and inclusive?
      options:
        - label: The venue is accessible
          required: false
        - label: Virtual attendance option available (if applicable)
          required: false
        - label: Content will be appropriate for diverse audiences
          required: true
