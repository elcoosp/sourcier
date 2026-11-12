import { defineCollection } from "astro:content";
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";
import { z } from "astro/zod";

export const collections = {
	docs: defineCollection({
		loader: docsLoader(),
		schema: docsSchema({
			// Merged into Starlight's default frontmatter schema.
			extend: z.object({
				/** Diátaxis information type of the page. */
				type: z
					.enum(["tutorial", "how-to", "reference", "explanation"])
					.optional(),
			}),
		}),
	}),
};
