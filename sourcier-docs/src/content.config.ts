import { defineCollection } from "astro:content";
import { docsLoader, i18nLoader } from "@astrojs/starlight/loaders";
import { docsSchema, i18nSchema } from "@astrojs/starlight/schema";
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
	i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
};
