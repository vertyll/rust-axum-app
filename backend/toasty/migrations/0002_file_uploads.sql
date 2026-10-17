ALTER TABLE "files" ADD COLUMN "object_key" TEXT;
ALTER TABLE "files" ADD COLUMN "status" TEXT;
UPDATE "files" SET "object_key" = "filename", "status" = 'uploaded';
ALTER TABLE "files" ALTER COLUMN "object_key" SET NOT NULL;
ALTER TABLE "files" ALTER COLUMN "status" SET NOT NULL;
CREATE INDEX "index_files_by_status" ON "files" ("status");
