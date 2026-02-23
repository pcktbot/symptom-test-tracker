export interface GlossaryEntry {
  name: string;
  panel: string;
  description: string;
  clinical: string;
}

export const GLOSSARY: GlossaryEntry[] = [
  // CBC
  { name: 'WBC', panel: 'CBC', description: 'White blood cell count. Measures the total number of infection-fighting cells in blood.', clinical: 'Elevated in infection, inflammation, or steroid use. Low in lupus flares, immunosuppressive therapy, or bone marrow suppression.' },
  { name: 'RBC', panel: 'CBC', description: 'Red blood cell count. Measures oxygen-carrying cells.', clinical: 'Low in anemia of chronic disease, common in lupus. May indicate hemolysis if paired with positive Coombs test.' },
  { name: 'Hemoglobin', panel: 'CBC', description: 'Protein in red blood cells that carries oxygen.', clinical: 'Low hemoglobin (anemia) is common in SLE. Can result from chronic inflammation, kidney disease, or autoimmune hemolytic anemia.' },
  { name: 'Hematocrit', panel: 'CBC', description: 'Percentage of blood volume occupied by red blood cells.', clinical: 'Tracks with hemoglobin. Low values confirm anemia.' },
  { name: 'MCV', panel: 'CBC', description: 'Mean corpuscular volume. Average size of red blood cells.', clinical: 'High MCV suggests B12/folate deficiency or certain medications. Low MCV suggests iron deficiency.' },
  { name: 'MCH', panel: 'CBC', description: 'Mean corpuscular hemoglobin. Average amount of hemoglobin per red blood cell.', clinical: 'Helps classify anemia type alongside MCV.' },
  { name: 'MCHC', panel: 'CBC', description: 'Mean corpuscular hemoglobin concentration. Average concentration of hemoglobin in red blood cells.', clinical: 'Low in iron deficiency anemia. Elevated in spherocytosis.' },
  { name: 'RDW', panel: 'CBC', description: 'Red cell distribution width. Measures variation in red blood cell size.', clinical: 'Elevated in iron deficiency, B12 deficiency, or mixed anemias. Can be high during active hemolysis.' },
  { name: 'Platelets', panel: 'CBC', description: 'Cell fragments essential for blood clotting.', clinical: 'Low platelets (thrombocytopenia) is common in lupus and a criterion for APS. Can indicate immune-mediated destruction.' },
  { name: 'Neutrophils', panel: 'CBC', description: 'Most abundant white blood cell type. First responders to bacterial infection.', clinical: 'Low in active lupus (neutropenia). Immunosuppressive drugs can also lower counts.' },
  { name: 'Lymphocytes', panel: 'CBC', description: 'White blood cells central to adaptive immunity (T cells, B cells).', clinical: 'Lymphopenia (<1000) is a diagnostic criterion for SLE and correlates with disease activity.' },
  { name: 'Monocytes', panel: 'CBC', description: 'White blood cells that become macrophages in tissues.', clinical: 'Generally less specific in lupus monitoring but can be elevated in chronic inflammation.' },
  { name: 'Eosinophils', panel: 'CBC', description: 'White blood cells involved in allergic responses and parasitic infections.', clinical: 'Usually normal in lupus. Elevation may suggest drug reaction or concurrent allergic condition.' },
  { name: 'Basophils', panel: 'CBC', description: 'Rarest white blood cell type. Involved in allergic and inflammatory responses.', clinical: 'Rarely significant in lupus monitoring.' },

  // Lipids
  { name: 'Total Cholesterol', panel: 'Lipids', description: 'Sum of all cholesterol types in blood.', clinical: 'Lupus and chronic inflammation can affect lipid metabolism. Steroids often raise cholesterol.' },
  { name: 'LDL', panel: 'Lipids', description: 'Low-density lipoprotein. "Bad" cholesterol that builds up in artery walls.', clinical: 'Often elevated by corticosteroid use. APS patients have increased cardiovascular risk, making LDL management important.' },
  { name: 'HDL', panel: 'Lipids', description: 'High-density lipoprotein. "Good" cholesterol that helps remove LDL.', clinical: 'Often low in active lupus. Low HDL combined with APS increases thrombotic risk.' },
  { name: 'Triglycerides', panel: 'Lipids', description: 'Fat molecules in the blood, major energy source.', clinical: 'Can be elevated by steroids, kidney disease, or metabolic syndrome in lupus patients.' },
  { name: 'VLDL', panel: 'Lipids', description: 'Very low-density lipoprotein. Carries triglycerides from liver to tissues.', clinical: 'Calculated from triglycerides. Elevated alongside triglycerides.' },
  { name: 'Non-HDL', panel: 'Lipids', description: 'Total cholesterol minus HDL. Captures all atherogenic lipoproteins.', clinical: 'Better predictor of cardiovascular risk than LDL alone, especially relevant for APS patients.' },

  // Inflammation
  { name: 'CRP', panel: 'Inflammation', description: 'C-reactive protein. Acute-phase protein produced by the liver in response to inflammation.', clinical: 'Useful for tracking infection and inflammation. In lupus, CRP may be only mildly elevated during flares (unlike in infection where it rises sharply).' },
  { name: 'ESR', panel: 'Inflammation', description: 'Erythrocyte sedimentation rate. Measures how quickly red blood cells settle in a tube.', clinical: 'Elevated in active lupus and tracks with disease activity. Less specific than CRP but useful for trend monitoring.' },
  { name: 'Ferritin', panel: 'Inflammation', description: 'Iron storage protein. Also an acute-phase reactant.', clinical: 'Can be elevated in inflammation regardless of iron status. Very high levels may suggest macrophage activation syndrome, a serious lupus complication.' },
  { name: 'Fibrinogen', panel: 'Inflammation', description: 'Clotting protein and acute-phase reactant.', clinical: 'Elevated in inflammation. Consumed during active clotting (DIC), which can occur in severe APS.' },
  { name: 'IL-6', panel: 'Inflammation', description: 'Interleukin-6. Pro-inflammatory cytokine.', clinical: 'Elevated in active lupus. Drives CRP production and B cell activation. Not routinely tested but useful for research or refractory cases.' },
  { name: 'Procalcitonin', panel: 'Inflammation', description: 'Precursor to calcitonin hormone. Rises specifically in bacterial infection.', clinical: 'Helps distinguish infection from lupus flare — procalcitonin rises in infection but typically stays low in flares.' },

  // Lupus/APS Markers
  { name: 'ANA', panel: 'Lupus/APS Markers', description: 'Antinuclear antibodies. Autoantibodies that target components of cell nuclei.', clinical: 'Positive in >95% of lupus patients. Reported as titer (1:80, 1:160, etc.) and pattern (homogeneous, speckled, nucleolar). High titers are more specific. A negative ANA nearly rules out SLE.' },
  { name: 'Anti-dsDNA', panel: 'Lupus/APS Markers', description: 'Antibodies against double-stranded DNA.', clinical: 'Highly specific for SLE. Levels correlate with disease activity, especially lupus nephritis. Rising titers often predict flares.' },
  { name: 'Anti-Smith', panel: 'Lupus/APS Markers', description: 'Antibodies against Smith antigen (snRNP proteins).', clinical: 'Highly specific for SLE (99%) but only present in 20-30% of patients. Levels do not correlate with disease activity.' },
  { name: 'Anti-SSA/Ro', panel: 'Lupus/APS Markers', description: 'Antibodies against Ro/SSA ribonucleoprotein.', clinical: 'Associated with photosensitivity, neonatal lupus, and Sjogren syndrome overlap. Important in pregnancy planning — can cause congenital heart block.' },
  { name: 'Anti-SSB/La', panel: 'Lupus/APS Markers', description: 'Antibodies against La/SSB ribonucleoprotein.', clinical: 'Usually appears with Anti-SSA. Strong association with Sjogren syndrome. Less common in SLE alone.' },
  { name: 'Anti-Scl-70', panel: 'Lupus/APS Markers', description: 'Antibodies against topoisomerase I.', clinical: 'Primarily associated with diffuse systemic sclerosis. Presence in lupus may suggest overlap syndrome.' },
  { name: 'Anti-Jo-1', panel: 'Lupus/APS Markers', description: 'Antibodies against histidyl-tRNA synthetase.', clinical: 'Associated with antisynthetase syndrome and polymyositis. Presence suggests myositis overlap.' },
  { name: 'C3', panel: 'Lupus/APS Markers', description: 'Complement component 3. Part of the innate immune system that enhances pathogen clearance.', clinical: 'Low C3 indicates complement consumption from active immune complex disease. Tracks closely with lupus nephritis activity.' },
  { name: 'C4', panel: 'Lupus/APS Markers', description: 'Complement component 4. Early classical pathway protein.', clinical: 'Often the first complement to drop in lupus flares. Some patients have genetic C4 deficiency (C4A null allele), causing a persistently low baseline.' },
  { name: 'CH50', panel: 'Lupus/APS Markers', description: 'Total hemolytic complement. Functional assay of the entire classical complement pathway.', clinical: 'Low when any complement component is deficient or consumed. Useful for confirming complement activation in active lupus.' },
  { name: 'Anticardiolipin IgG', panel: 'Lupus/APS Markers', description: 'IgG antibodies against cardiolipin, a phospholipid in cell membranes.', clinical: 'One of the three criteria antibodies for antiphospholipid syndrome. Must be present at medium-to-high titer. Persistent positivity (12+ weeks apart) is required for APS diagnosis.' },
  { name: 'Anticardiolipin IgM', panel: 'Lupus/APS Markers', description: 'IgM antibodies against cardiolipin.', clinical: 'Part of APS criteria. IgM is less specific than IgG and can be transiently positive in infections.' },
  { name: 'Beta-2 Glycoprotein I IgG', panel: 'Lupus/APS Markers', description: 'IgG antibodies against beta-2 glycoprotein I, a phospholipid-binding protein.', clinical: 'Considered the most specific antiphospholipid antibody for APS. Strong association with thrombotic events.' },
  { name: 'Beta-2 Glycoprotein I IgM', panel: 'Lupus/APS Markers', description: 'IgM antibodies against beta-2 glycoprotein I.', clinical: 'Part of the APS antibody panel. Less specific than IgG but still included in diagnostic criteria.' },
  { name: 'Lupus Anticoagulant', panel: 'Lupus/APS Markers', description: 'Functional coagulation assay detecting antibodies that prolong phospholipid-dependent clotting in vitro.', clinical: 'Paradoxically, despite prolonging clotting tests in the lab, it is the strongest predictor of thrombosis among APS antibodies. Reported as positive/negative.' },
  { name: 'Direct Coombs', panel: 'Lupus/APS Markers', description: 'Detects antibodies or complement bound to the surface of red blood cells.', clinical: 'Positive in autoimmune hemolytic anemia (AIHA), which can occur in SLE. A positive Coombs without overt hemolysis counts as a diagnostic criterion.' },
  { name: 'Urine Protein/Creatinine', panel: 'Lupus/APS Markers', description: 'Ratio of protein to creatinine in a spot urine sample. Estimates 24-hour protein excretion.', clinical: 'Key marker for lupus nephritis. Ratio >0.5 (500 mg/g) suggests significant proteinuria. Serial monitoring guides treatment response.' },

  // Metabolic / BMP
  { name: 'Sodium', panel: 'Metabolic / BMP', description: 'Electrolyte critical for fluid balance, nerve, and muscle function.', clinical: 'Can be low (SIADH) in active lupus or from certain medications.' },
  { name: 'Potassium', panel: 'Metabolic / BMP', description: 'Electrolyte essential for heart rhythm and muscle contraction.', clinical: 'Monitor closely with kidney disease or when on ACE inhibitors, ARBs, or certain immunosuppressants.' },
  { name: 'Chloride', panel: 'Metabolic / BMP', description: 'Electrolyte that helps maintain acid-base balance.', clinical: 'Usually tracks with sodium. Abnormal in acid-base disorders.' },
  { name: 'CO2', panel: 'Metabolic / BMP', description: 'Bicarbonate level. Reflects acid-base status.', clinical: 'Low in metabolic acidosis, which can occur with kidney disease in lupus nephritis.' },
  { name: 'BUN', panel: 'Metabolic / BMP', description: 'Blood urea nitrogen. Waste product filtered by kidneys.', clinical: 'Elevated in kidney dysfunction. Important for monitoring lupus nephritis.' },
  { name: 'Creatinine', panel: 'Metabolic / BMP', description: 'Waste product from muscle metabolism, filtered by kidneys.', clinical: 'Most important single marker for kidney function. Rising creatinine in lupus may indicate nephritis progression.' },
  { name: 'eGFR', panel: 'Metabolic / BMP', description: 'Estimated glomerular filtration rate. Calculated measure of kidney filtering capacity.', clinical: 'Below 60 indicates chronic kidney disease. Serial monitoring is critical in lupus nephritis.' },
  { name: 'Glucose', panel: 'Metabolic / BMP', description: 'Blood sugar level.', clinical: 'Corticosteroids commonly cause elevated glucose. Monitor for steroid-induced diabetes.' },
  { name: 'Calcium', panel: 'Metabolic / BMP', description: 'Mineral essential for bones, muscle contraction, and nerve signaling.', clinical: 'Can be affected by kidney disease or vitamin D deficiency, both common in lupus.' },
  { name: 'ALT', panel: 'Metabolic / BMP', description: 'Alanine aminotransferase. Liver enzyme most specific for liver cell damage.', clinical: 'Monitor for hepatotoxicity from methotrexate, azathioprine, or other immunosuppressants.' },
  { name: 'AST', panel: 'Metabolic / BMP', description: 'Aspartate aminotransferase. Enzyme found in liver and muscle.', clinical: 'Elevated alongside ALT in drug-induced liver injury. Can also rise from muscle inflammation (myositis).' },
  { name: 'Alkaline Phosphatase', panel: 'Metabolic / BMP', description: 'Enzyme found in liver, bone, and bile ducts.', clinical: 'Elevated in cholestatic liver disease. Less commonly affected by typical lupus medications.' },
  { name: 'Total Bilirubin', panel: 'Metabolic / BMP', description: 'Breakdown product of hemoglobin. Processed by the liver.', clinical: 'Elevated in hemolytic anemia (common in lupus) or liver dysfunction.' },
  { name: 'Albumin', panel: 'Metabolic / BMP', description: 'Major blood protein made by the liver. Maintains blood volume and carries substances.', clinical: 'Low in nephrotic syndrome from lupus nephritis (lost in urine), chronic inflammation, or liver disease.' },

  // Thyroid
  { name: 'TSH', panel: 'Thyroid', description: 'Thyroid-stimulating hormone. Pituitary hormone that controls thyroid function.', clinical: 'Autoimmune thyroid disease (Hashimoto\'s) frequently co-occurs with lupus. High TSH indicates hypothyroidism.' },
  { name: 'Free T4', panel: 'Thyroid', description: 'Free thyroxine. Active thyroid hormone available to tissues.', clinical: 'Low with high TSH confirms hypothyroidism. Thyroid function should be checked periodically in lupus patients.' },
  { name: 'Free T3', panel: 'Thyroid', description: 'Free triiodothyronine. Most active thyroid hormone.', clinical: 'Can be low in chronic illness ("sick euthyroid") even with normal TSH. Less useful for routine monitoring.' },
  { name: 'TPO Antibodies', panel: 'Thyroid', description: 'Thyroid peroxidase antibodies. Autoantibodies against thyroid enzyme.', clinical: 'Positive in Hashimoto\'s thyroiditis. Lupus patients have a higher prevalence of thyroid autoimmunity.' },

  // Coagulation
  { name: 'PT', panel: 'Coagulation', description: 'Prothrombin time. Measures extrinsic and common clotting pathway function.', clinical: 'Can be prolonged by warfarin therapy (common in APS) or liver disease.' },
  { name: 'INR', panel: 'Coagulation', description: 'International normalized ratio. Standardized PT result for monitoring anticoagulation.', clinical: 'Target INR for APS patients on warfarin is typically 2.0-3.0 (or 2.5-3.5 for arterial events). Critical for preventing recurrent thrombosis.' },
  { name: 'aPTT', panel: 'Coagulation', description: 'Activated partial thromboplastin time. Measures intrinsic and common clotting pathway.', clinical: 'Paradoxically prolonged by lupus anticoagulant in vitro despite increased clotting risk in vivo. Used as part of lupus anticoagulant testing.' },
];

import type { CustomLabTest } from './types';
import { getCustomLabTests } from './db';

export function getGlossaryEntry(testName: string): GlossaryEntry | undefined {
  return GLOSSARY.find(e => e.name === testName);
}

export function getGlossaryByPanel(): Record<string, GlossaryEntry[]> {
  const groups: Record<string, GlossaryEntry[]> = {};
  for (const entry of GLOSSARY) {
    if (!groups[entry.panel]) groups[entry.panel] = [];
    groups[entry.panel].push(entry);
  }
  return groups;
}

export async function getMergedGlossary(): Promise<Record<string, GlossaryEntry[]>> {
  const groups = getGlossaryByPanel();
  const custom = await getCustomLabTests();
  for (const ct of custom) {
    if (!ct.description && !ct.clinical) continue;
    if (!groups[ct.panel]) groups[ct.panel] = [];
    // Only add if not already present
    if (!groups[ct.panel].some(e => e.name === ct.name)) {
      groups[ct.panel].push({
        name: ct.name,
        panel: ct.panel,
        description: ct.description,
        clinical: ct.clinical,
      });
    }
  }
  return groups;
}
