import { spawn } from "child_process";
import path from "path";
import { logger } from "../logger";

export class ComplianceService {
  /**
   * Triggers the compliance report generation script.
   * Returns a promise that resolves with the report URL if successful.
   */
  async triggerReport(): Promise<string> {
    return new Promise((resolve, reject) => {
      const scriptPath = path.resolve(__dirname, "../../../scripts/generate_compliance_report.sh");
      const year = new Date().getFullYear().toString();
      const month = (new Date().getMonth() + 1).toString().padStart(2, "0");
      
      logger.info("Triggering compliance report generation", { scriptPath, year, month });

      const child = spawn("bash", [scriptPath], {
        env: {
          ...process.env,
          YEAR: year,
          MONTH: month,
        },
      });

      let output = "";
      child.stdout.on("data", (data) => {
        output += data.toString();
      });

      child.stderr.on("data", (data) => {
        logger.error("Compliance script error output", { stderr: data.toString() });
      });

      child.on("close", (code) => {
        if (code === 0) {
          // Parse output for the S3 URL (assuming the script logs it at the end)
          const match = output.match(/s3:\/\/[\w.-]+\/.+\.pdf/);
          const reportUrl = match ? match[0] : `s3://streminderminds-compliance-reports/${year}/${month}/report_${year}_${month}.pdf`;
          
          logger.info("Compliance report generated successfully", { reportUrl });
          resolve(reportUrl);
        } else {
          logger.error("Compliance report generation failed", { exitCode: code });
          reject(new Error(`Compliance script failed with exit code ${code}`));
        }
      });
    });
  }
}

export const complianceService = new ComplianceService();
