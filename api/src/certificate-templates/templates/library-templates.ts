import { TemplateCategory, TemplateLayout } from "../entities/certificate-template.entity";

const a4Landscape = (overrides: Partial<TemplateLayout> = {}): TemplateLayout => ({
  width: 297, height: 210, orientation: "landscape",
  background: { type: "color", value: "#FFFFFF" },
  elements: [], ...overrides,
});

const PALETTES = [
  { primary: "#1A237E", accent: "#7986CB", bg: "#E8EAF6" },
  { primary: "#004D40", accent: "#4DB6AC", bg: "#E0F2F1" },
  { primary: "#BF360C", accent: "#FF8A65", bg: "#FBE9E7" },
  { primary: "#4A148C", accent: "#CE93D8", bg: "#F3E5F5" },
  { primary: "#0D47A1", accent: "#64B5F6", bg: "#E3F2FD" },
  { primary: "#1B5E20", accent: "#81C784", bg: "#E8F5E9" },
  { primary: "#F57F17", accent: "#FFD54F", bg: "#FFFDE7" },
  { primary: "#37474F", accent: "#90A4AE", bg: "#ECEFF1" },
  { primary: "#880E4F", accent: "#F48FB1", bg: "#FCE4EC" },
  { primary: "#006064", accent: "#4DD0E1", bg: "#E0F7FA" },
];

const baseElements = (p: { primary: string; accent: string }) => [
  { id: "border", type: "shape" as const, x: 8, y: 8, width: 281, height: 194, zIndex: 0,
    properties: { shape: "rectangle", borderColor: p.primary, borderWidth: 3, fill: "none" } },
  { id: "title", type: "text" as const, x: 50, y: 20, width: 197, height: 20, zIndex: 1,
    properties: { text: "CERTIFICATE OF {{type}}", fontSize: 22, fontWeight: "bold", color: p.primary, align: "center", fontFamily: "Playfair Display" } },
  { id: "subtitle", type: "text" as const, x: 50, y: 44, width: 197, height: 10, zIndex: 1,
    properties: { text: "This is to certify that", fontSize: 11, color: "#555555", align: "center", fontFamily: "Open Sans" } },
  { id: "recipient", type: "text" as const, x: 50, y: 58, width: 197, height: 16, zIndex: 1,
    properties: { text: "{{recipientName}}", fontSize: 28, fontWeight: "bold", fontStyle: "italic", color: p.accent, align: "center", fontFamily: "Great Vibes", placeholder: true } },
  { id: "body", type: "text" as const, x: 50, y: 80, width: 197, height: 24, zIndex: 1,
    properties: { text: "has successfully completed\n{{courseName}}", fontSize: 13, color: "#333333", align: "center", fontFamily: "Open Sans", placeholder: true } },
  { id: "date-label", type: "text" as const, x: 30, y: 145, width: 80, height: 8, zIndex: 1,
    properties: { text: "Date", fontSize: 9, color: "#888888", align: "center", fontFamily: "Open Sans" } },
  { id: "date-value", type: "text" as const, x: 30, y: 155, width: 80, height: 8, zIndex: 1,
    properties: { text: "{{completionDate}}", fontSize: 11, color: p.primary, align: "center", fontFamily: "Open Sans", fontWeight: "bold", placeholder: true } },
  { id: "date-line", type: "shape" as const, x: 30, y: 163, width: 80, height: 1, zIndex: 1,
    properties: { shape: "line", color: p.primary } },
  { id: "sig-label", type: "text" as const, x: 187, y: 145, width: 80, height: 8, zIndex: 1,
    properties: { text: "Instructor", fontSize: 9, color: "#888888", align: "center", fontFamily: "Open Sans" } },
  { id: "sig-value", type: "text" as const, x: 187, y: 155, width: 80, height: 8, zIndex: 1,
    properties: { text: "{{instructorName}}", fontSize: 11, color: p.primary, align: "center", fontFamily: "Open Sans", fontWeight: "bold", placeholder: true } },
  { id: "sig-line", type: "shape" as const, x: 187, y: 163, width: 80, height: 1, zIndex: 1,
    properties: { shape: "line", color: p.primary } },
  { id: "qrcode", type: "qrcode" as const, x: 130, y: 145, width: 30, height: 30, zIndex: 2,
    properties: { data: "{{verificationUrl}}", foregroundColor: p.primary, backgroundColor: "#FFFFFF", placeholder: true } },
  { id: "accent-bar", type: "shape" as const, x: 8, y: 16, width: 4, height: 178, zIndex: 0,
    properties: { shape: "rectangle", fill: p.accent, borderWidth: 0 } },
];

interface LibrarySeed {
  name: string; description: string;
  category: TemplateCategory; tags: string[];
  palette: number; titleType: string;
}

const seeds: LibrarySeed[] = [
  { name: "Classic Blue Completion", description: "Timeless blue completion certificate", category: "completion", tags: ["classic", "blue"], palette: 0, titleType: "COMPLETION" },
  { name: "Forest Green Completion", description: "Nature-inspired completion certificate", category: "completion", tags: ["green", "nature"], palette: 1, titleType: "COMPLETION" },
  { name: "Coral Flame Completion", description: "Bold coral completion template", category: "completion", tags: ["coral", "bold"], palette: 2, titleType: "COMPLETION" },
  { name: "Royal Purple Completion", description: "Elegant purple completion certificate", category: "completion", tags: ["purple", "elegant"], palette: 3, titleType: "COMPLETION" },
  { name: "Ocean Blue Completion", description: "Deep ocean blue completion template", category: "completion", tags: ["ocean", "blue"], palette: 4, titleType: "COMPLETION" },
  { name: "Emerald Completion", description: "Emerald green completion certificate", category: "completion", tags: ["emerald", "green"], palette: 5, titleType: "COMPLETION" },
  { name: "Golden Amber Completion", description: "Warm amber tones completion template", category: "completion", tags: ["gold", "warm"], palette: 6, titleType: "COMPLETION" },
  { name: "Slate Grey Completion", description: "Professional slate completion certificate", category: "completion", tags: ["grey", "professional"], palette: 7, titleType: "COMPLETION" },
  { name: "Rose Completion", description: "Elegant rose completion template", category: "completion", tags: ["rose", "elegant"], palette: 8, titleType: "COMPLETION" },
  { name: "Teal Completion", description: "Fresh teal completion certificate", category: "completion", tags: ["teal", "fresh"], palette: 9, titleType: "COMPLETION" },
  { name: "Navy Classic Completion", description: "Navy blue formal completion", category: "completion", tags: ["navy", "formal"], palette: 0, titleType: "COMPLETION" },
  { name: "Sage Green Completion", description: "Calming sage completion certificate", category: "completion", tags: ["sage", "calm"], palette: 1, titleType: "COMPLETION" },
  { name: "Crimson Completion", description: "Bold crimson completion template", category: "completion", tags: ["crimson", "bold"], palette: 2, titleType: "COMPLETION" },
  { name: "Lavender Completion", description: "Soft lavender completion certificate", category: "completion", tags: ["lavender", "soft"], palette: 3, titleType: "COMPLETION" },
  { name: "Sky Completion", description: "Light sky blue completion template", category: "completion", tags: ["sky", "light"], palette: 4, titleType: "COMPLETION" },
  { name: "Mint Completion", description: "Fresh mint green completion certificate", category: "completion", tags: ["mint", "fresh"], palette: 5, titleType: "COMPLETION" },
  { name: "Sunshine Completion", description: "Bright sunshine completion template", category: "completion", tags: ["yellow", "bright"], palette: 6, titleType: "COMPLETION" },
  { name: "Charcoal Completion", description: "Modern charcoal completion certificate", category: "completion", tags: ["charcoal", "modern"], palette: 7, titleType: "COMPLETION" },
  { name: "Blush Completion", description: "Delicate blush completion template", category: "completion", tags: ["blush", "delicate"], palette: 8, titleType: "COMPLETION" },
  { name: "Aqua Completion", description: "Vibrant aqua completion certificate", category: "completion", tags: ["aqua", "vibrant"], palette: 9, titleType: "COMPLETION" },
  { name: "Indigo Completion", description: "Deep indigo completion template", category: "completion", tags: ["indigo", "deep"], palette: 0, titleType: "COMPLETION" },
  { name: "Olive Completion", description: "Earthy olive completion certificate", category: "completion", tags: ["olive", "earthy"], palette: 1, titleType: "COMPLETION" },
  { name: "Rust Completion", description: "Warm rust completion template", category: "completion", tags: ["rust", "warm"], palette: 2, titleType: "COMPLETION" },
  { name: "Plum Completion", description: "Rich plum completion certificate", category: "completion", tags: ["plum", "rich"], palette: 3, titleType: "COMPLETION" },
  { name: "Cobalt Completion", description: "Striking cobalt completion template", category: "completion", tags: ["cobalt", "striking"], palette: 4, titleType: "COMPLETION" },
  { name: "Pine Completion", description: "Classic pine green completion certificate", category: "completion", tags: ["pine", "classic"], palette: 5, titleType: "COMPLETION" },
  { name: "Honey Completion", description: "Warm honey completion template", category: "completion", tags: ["honey", "warm"], palette: 6, titleType: "COMPLETION" },
  { name: "Pewter Completion", description: "Refined pewter completion certificate", category: "completion", tags: ["pewter", "refined"], palette: 7, titleType: "COMPLETION" },
  { name: "Mauve Completion", description: "Soft mauve completion template", category: "completion", tags: ["mauve", "soft"], palette: 8, titleType: "COMPLETION" },
  { name: "Cyan Completion", description: "Crisp cyan completion certificate", category: "completion", tags: ["cyan", "crisp"], palette: 9, titleType: "COMPLETION" },
  { name: "Gold Achievement", description: "Premium gold achievement certificate", category: "achievement", tags: ["gold", "premium"], palette: 6, titleType: "ACHIEVEMENT" },
  { name: "Blue Achievement", description: "Bold blue achievement certificate", category: "achievement", tags: ["blue", "bold"], palette: 0, titleType: "ACHIEVEMENT" },
  { name: "Green Achievement", description: "Verdant achievement certificate", category: "achievement", tags: ["green"], palette: 1, titleType: "ACHIEVEMENT" },
  { name: "Red Achievement", description: "Dynamic red achievement template", category: "achievement", tags: ["red", "dynamic"], palette: 2, titleType: "ACHIEVEMENT" },
  { name: "Purple Achievement", description: "Majestic purple achievement certificate", category: "achievement", tags: ["purple", "majestic"], palette: 3, titleType: "ACHIEVEMENT" },
  { name: "Teal Achievement", description: "Cool teal achievement template", category: "achievement", tags: ["teal", "cool"], palette: 9, titleType: "ACHIEVEMENT" },
  { name: "Grey Achievement", description: "Professional grey achievement certificate", category: "achievement", tags: ["grey", "professional"], palette: 7, titleType: "ACHIEVEMENT" },
  { name: "Pink Achievement", description: "Elegant pink achievement template", category: "achievement", tags: ["pink", "elegant"], palette: 8, titleType: "ACHIEVEMENT" },
  { name: "Navy Achievement", description: "Authoritative navy achievement certificate", category: "achievement", tags: ["navy"], palette: 0, titleType: "ACHIEVEMENT" },
  { name: "Forest Achievement", description: "Rich forest achievement template", category: "achievement", tags: ["forest", "rich"], palette: 1, titleType: "ACHIEVEMENT" },
  { name: "Flame Achievement", description: "High-energy flame achievement certificate", category: "achievement", tags: ["flame", "energy"], palette: 2, titleType: "ACHIEVEMENT" },
  { name: "Violet Achievement", description: "Deep violet achievement template", category: "achievement", tags: ["violet", "deep"], palette: 3, titleType: "ACHIEVEMENT" },
  { name: "Azure Achievement", description: "Clear azure achievement certificate", category: "achievement", tags: ["azure", "clear"], palette: 4, titleType: "ACHIEVEMENT" },
  { name: "Jade Achievement", description: "Precious jade achievement template", category: "achievement", tags: ["jade", "precious"], palette: 5, titleType: "ACHIEVEMENT" },
  { name: "Amber Achievement", description: "Warm amber achievement certificate", category: "achievement", tags: ["amber", "warm"], palette: 6, titleType: "ACHIEVEMENT" },
  { name: "Steel Achievement", description: "Modern steel achievement template", category: "achievement", tags: ["steel", "modern"], palette: 7, titleType: "ACHIEVEMENT" },
  { name: "Fuchsia Achievement", description: "Vibrant fuchsia achievement certificate", category: "achievement", tags: ["fuchsia", "vibrant"], palette: 8, titleType: "ACHIEVEMENT" },
  { name: "Turquoise Achievement", description: "Tropical turquoise achievement template", category: "achievement", tags: ["turquoise"], palette: 9, titleType: "ACHIEVEMENT" },
  { name: "Bronze Achievement", description: "Classic bronze achievement certificate", category: "achievement", tags: ["bronze", "classic"], palette: 0, titleType: "ACHIEVEMENT" },
  { name: "Silver Achievement", description: "Sleek silver achievement template", category: "achievement", tags: ["silver", "sleek"], palette: 7, titleType: "ACHIEVEMENT" },
  { name: "Platinum Excellence", description: "Top-tier platinum excellence certificate", category: "excellence", tags: ["platinum", "top-tier"], palette: 7, titleType: "EXCELLENCE" },
  { name: "Diamond Excellence", description: "Premium diamond excellence certificate", category: "excellence", tags: ["diamond", "premium"], palette: 4, titleType: "EXCELLENCE" },
  { name: "Gold Excellence", description: "Classic gold excellence template", category: "excellence", tags: ["gold", "classic"], palette: 6, titleType: "EXCELLENCE" },
  { name: "Royal Excellence", description: "Regal royal excellence certificate", category: "excellence", tags: ["royal", "regal"], palette: 3, titleType: "EXCELLENCE" },
  { name: "Navy Excellence", description: "Distinguished navy excellence template", category: "excellence", tags: ["navy", "distinguished"], palette: 0, titleType: "EXCELLENCE" },
  { name: "Emerald Excellence", description: "Prestige emerald excellence certificate", category: "excellence", tags: ["emerald", "prestige"], palette: 5, titleType: "EXCELLENCE" },
  { name: "Crimson Excellence", description: "Bold crimson excellence template", category: "excellence", tags: ["crimson", "bold"], palette: 2, titleType: "EXCELLENCE" },
  { name: "Sapphire Excellence", description: "Deep sapphire excellence certificate", category: "excellence", tags: ["sapphire", "deep"], palette: 4, titleType: "EXCELLENCE" },
  { name: "Onyx Excellence", description: "Sleek onyx excellence template", category: "excellence", tags: ["onyx", "sleek"], palette: 7, titleType: "EXCELLENCE" },
  { name: "Coral Excellence", description: "Vibrant coral excellence certificate", category: "excellence", tags: ["coral", "vibrant"], palette: 2, titleType: "EXCELLENCE" },
  { name: "Teal Excellence", description: "Elegant teal excellence template", category: "excellence", tags: ["teal", "elegant"], palette: 9, titleType: "EXCELLENCE" },
  { name: "Lavender Excellence", description: "Graceful lavender excellence certificate", category: "excellence", tags: ["lavender", "graceful"], palette: 3, titleType: "EXCELLENCE" },
  { name: "Cobalt Excellence", description: "Striking cobalt excellence template", category: "excellence", tags: ["cobalt", "striking"], palette: 4, titleType: "EXCELLENCE" },
  { name: "Forest Excellence", description: "Deep forest excellence certificate", category: "excellence", tags: ["forest", "deep"], palette: 1, titleType: "EXCELLENCE" },
  { name: "Ruby Excellence", description: "Precious ruby excellence template", category: "excellence", tags: ["ruby", "precious"], palette: 2, titleType: "EXCELLENCE" },
  { name: "Indigo Excellence", description: "Mystic indigo excellence certificate", category: "excellence", tags: ["indigo", "mystic"], palette: 3, titleType: "EXCELLENCE" },
  { name: "Steel Excellence", description: "Refined steel excellence template", category: "excellence", tags: ["steel", "refined"], palette: 7, titleType: "EXCELLENCE" },
  { name: "Rose Excellence", description: "Delicate rose excellence certificate", category: "excellence", tags: ["rose", "delicate"], palette: 8, titleType: "EXCELLENCE" },
  { name: "Aqua Excellence", description: "Clear aqua excellence template", category: "excellence", tags: ["aqua", "clear"], palette: 9, titleType: "EXCELLENCE" },
  { name: "Amber Excellence", description: "Warm amber excellence certificate", category: "excellence", tags: ["amber", "warm"], palette: 6, titleType: "EXCELLENCE" },
  { name: "Classic Participation", description: "Standard participation certificate", category: "participation", tags: ["classic", "standard"], palette: 0, titleType: "PARTICIPATION" },
  { name: "Warm Participation", description: "Warm tones participation certificate", category: "participation", tags: ["warm"], palette: 6, titleType: "PARTICIPATION" },
  { name: "Cool Participation", description: "Cool blue participation certificate", category: "participation", tags: ["cool", "blue"], palette: 4, titleType: "PARTICIPATION" },
  { name: "Green Participation", description: "Fresh green participation certificate", category: "participation", tags: ["green", "fresh"], palette: 5, titleType: "PARTICIPATION" },
  { name: "Purple Participation", description: "Creative purple participation certificate", category: "participation", tags: ["purple", "creative"], palette: 3, titleType: "PARTICIPATION" },
  { name: "Teal Participation", description: "Modern teal participation certificate", category: "participation", tags: ["teal", "modern"], palette: 9, titleType: "PARTICIPATION" },
  { name: "Red Participation", description: "Energetic red participation certificate", category: "participation", tags: ["red", "energetic"], palette: 2, titleType: "PARTICIPATION" },
  { name: "Grey Participation", description: "Neutral grey participation certificate", category: "participation", tags: ["grey", "neutral"], palette: 7, titleType: "PARTICIPATION" },
  { name: "Rose Participation", description: "Soft rose participation certificate", category: "participation", tags: ["rose", "soft"], palette: 8, titleType: "PARTICIPATION" },
  { name: "Navy Participation", description: "Formal navy participation certificate", category: "participation", tags: ["navy", "formal"], palette: 0, titleType: "PARTICIPATION" },
  { name: "Lime Participation", description: "Bright lime participation certificate", category: "participation", tags: ["lime", "bright"], palette: 1, titleType: "PARTICIPATION" },
  { name: "Orange Participation", description: "Energizing orange participation certificate", category: "participation", tags: ["orange"], palette: 6, titleType: "PARTICIPATION" },
  { name: "Midnight Participation", description: "Deep midnight participation certificate", category: "participation", tags: ["midnight"], palette: 4, titleType: "PARTICIPATION" },
  { name: "Sage Participation", description: "Calming sage participation certificate", category: "participation", tags: ["sage", "calm"], palette: 5, titleType: "PARTICIPATION" },
  { name: "Blush Participation", description: "Gentle blush participation certificate", category: "participation", tags: ["blush", "gentle"], palette: 8, titleType: "PARTICIPATION" },
  { name: "Executive Professional", description: "Top-tier executive professional certificate", category: "professional", tags: ["executive", "formal"], palette: 7, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Corporate Blue Professional", description: "Corporate blue professional certificate", category: "professional", tags: ["corporate", "blue"], palette: 0, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Management Professional", description: "Leadership management professional certificate", category: "professional", tags: ["management", "leadership"], palette: 4, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Technical Professional", description: "Technical skills professional certificate", category: "professional", tags: ["technical", "skills"], palette: 9, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Finance Professional", description: "Finance industry professional certificate", category: "professional", tags: ["finance", "industry"], palette: 5, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Law Professional", description: "Legal domain professional certificate", category: "professional", tags: ["legal", "law"], palette: 3, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Medical Professional", description: "Healthcare professional certificate", category: "professional", tags: ["healthcare", "medical"], palette: 1, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Engineering Professional", description: "Engineering discipline professional certificate", category: "professional", tags: ["engineering"], palette: 4, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Design Professional", description: "Creative design professional certificate", category: "professional", tags: ["design", "creative"], palette: 8, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Education Professional", description: "Education sector professional certificate", category: "professional", tags: ["education"], palette: 6, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "IT Professional", description: "Information technology professional certificate", category: "professional", tags: ["IT", "technology"], palette: 0, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Marketing Professional", description: "Marketing professional certificate", category: "professional", tags: ["marketing"], palette: 2, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Blockchain Professional", description: "Blockchain industry professional certificate", category: "professional", tags: ["blockchain", "web3"], palette: 3, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Data Science Professional", description: "Data science professional certificate", category: "professional", tags: ["data", "analytics"], palette: 4, titleType: "PROFESSIONAL DEVELOPMENT" },
  { name: "Cybersecurity Professional", description: "Cybersecurity professional certificate", category: "professional", tags: ["security", "cyber"], palette: 7, titleType: "PROFESSIONAL DEVELOPMENT" },
];

export interface LibraryTemplateSeed {
  name: string; description: string; category: TemplateCategory;
  tags: string[]; isLibraryTemplate: true; qrCodeEnabled: true;
  printOptimized: true; layout: TemplateLayout;
}

export const LIBRARY_TEMPLATES: LibraryTemplateSeed[] = seeds.map((seed) => {
  const palette = PALETTES[seed.palette % PALETTES.length];
  const elements = baseElements(palette).map((el) =>
    el.id === "title"
      ? { ...el, properties: { ...el.properties, text: `CERTIFICATE OF ${seed.titleType}` } }
      : el
  );
  return {
    name: seed.name, description: seed.description,
    category: seed.category, tags: seed.tags,
    isLibraryTemplate: true as const, qrCodeEnabled: true as const, printOptimized: true as const,
    layout: a4Landscape({ background: { type: "color", value: palette.bg }, elements }),
  };
});
