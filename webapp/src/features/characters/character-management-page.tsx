import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { faCheckDouble } from "@fortawesome/free-solid-svg-icons/faCheckDouble";
import { faChevronLeft } from "@fortawesome/free-solid-svg-icons/faChevronLeft";
import { faChevronRight } from "@fortawesome/free-solid-svg-icons/faChevronRight";
import { faFolder } from "@fortawesome/free-solid-svg-icons/faFolder";
import { faFolderOpen } from "@fortawesome/free-solid-svg-icons/faFolderOpen";
import { faList } from "@fortawesome/free-solid-svg-icons/faList";
import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons/faMagnifyingGlass";
import { faPen } from "@fortawesome/free-solid-svg-icons/faPen";
import { faPlus } from "@fortawesome/free-solid-svg-icons/faPlus";
import { faSquareCheck } from "@fortawesome/free-solid-svg-icons/faSquareCheck";
import { faTableCellsLarge } from "@fortawesome/free-solid-svg-icons/faTableCellsLarge";
import { faTrashCan } from "@fortawesome/free-solid-svg-icons/faTrashCan";
import { faUpload } from "@fortawesome/free-solid-svg-icons/faUpload";
import { faWandMagicSparkles } from "@fortawesome/free-solid-svg-icons/faWandMagicSparkles";
import { faXmark } from "@fortawesome/free-solid-svg-icons/faXmark";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent, CSSProperties, ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../../components/ui/button";
import { IconButton } from "../../components/ui/icon-button";
import { Input } from "../../components/ui/input";
import { WorkspacePanelShell } from "../../components/layout/workspace-panel-shell";
import { useWorkspaceLayoutContext } from "../../components/layout/workspace-context";
import { Badge } from "../../components/ui/badge";
import {
	Dialog,
	DialogBody,
	DialogClose,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "../../components/ui/dialog";
import { InsertSampleDialog } from "../demo-content/insert-sample-dialog";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "../../components/ui/card";
import { SectionHeader } from "../../components/ui/section-header";
import { useToastNotice } from "../../components/ui/toast-context";
import { cn } from "../../lib/cn";
import { revokeObjectUrl } from "../../lib/binary-resource";
import { isRpcConflict } from "../../lib/rpc";
import { demoCharacterDefinitions, loadDemoCoverFile } from "../demo-content/konosuba-sample-data";
import { createSchema, listSchemas } from "../schemas/api";
import { buildSchemaPresetDefinitions } from "../schemas/schema-presets";
import {
	createCharacter,
	deleteCharacter,
	downloadCharacterArchive,
	getCharacter,
	getCharacterCoverUrl,
	hasCharacterCardExtension,
	importCharacterArchive,
	listCharacters,
	setCharacterCover,
	updateCharacter,
} from "./api";
import { CharacterFormDialog } from "./create-character-dialog";
import { DeleteCharacterDialog } from "./delete-character-dialog";
import { CharacterDetailsDialog } from "./character-details-dialog";
import {
	addCharacterFolderRegistryEntry,
	loadCharacterFolderRegistry,
	normalizeCharacterFolderRegistryName,
	removeCharacterFolderRegistryEntry,
	renameCharacterFolderRegistryEntry,
	saveCharacterFolderRegistry,
} from "./folder-registry";
import type { CharacterSummary } from "./types";

type NoticeTone = "error" | "success" | "warning";
type CharacterViewMode = "grid" | "list";

type Notice = {
	message: string;
	tone: NoticeTone;
};

type FolderDialogState =
	| {
			mode: "create";
	  }
	| {
			folder: string;
			mode: "rename";
	  };

type ContextMenuState =
	| {
			target: "root";
			x: number;
			y: number;
	  }
	| {
			folder: string;
			target: "folder";
			x: number;
			y: number;
	  };

const COVER_OBJECT_POSITION = "center 26%";
const CARD_EXCERPT_STYLE: CSSProperties = {
	WebkitBoxOrient: "vertical",
	WebkitLineClamp: 2,
	display: "-webkit-box",
	overflow: "hidden",
};
const LIST_EXCERPT_STYLE: CSSProperties = {
	overflow: "hidden",
	textOverflow: "ellipsis",
	whiteSpace: "nowrap",
};

function getErrorMessage(error: unknown, fallback: string) {
	return error instanceof Error ? error.message : fallback;
}

function getCharacterMonogram(name: string) {
	return Array.from(name.trim())[0] ?? "?";
}

function normalizeSummaryText(text: string) {
	return text.replace(/\s+/g, " ").trim();
}

function normalizeCharacterFolder(folder: string) {
	return normalizeCharacterFolderRegistryName(folder);
}

function getCharacterFolderLabel(folder: string, unfiledLabel: string) {
	return folder || unfiledLabel;
}

function buildCharacterSearchText(summary: CharacterSummary) {
	return [
		summary.character_id,
		summary.name,
		summary.personality,
		summary.style,
		normalizeCharacterFolder(summary.folder),
		...summary.tags,
	]
		.join(" ")
		.toLocaleLowerCase();
}

function FolderTreeItem({
	active,
	count,
	dragActive,
	icon,
	label,
	onClick,
	onContextMenu,
	onDragLeave,
	onDragOver,
	onDrop,
}: {
	active: boolean;
	count?: number;
	dragActive?: boolean;
	icon: ReactNode;
	label: string;
	onClick: () => void;
	onContextMenu?: (event: React.MouseEvent<HTMLButtonElement>) => void;
	onDragLeave?: () => void;
	onDragOver?: (event: React.DragEvent<HTMLButtonElement>) => void;
	onDrop?: (event: React.DragEvent<HTMLButtonElement>) => void;
}) {
	return (
		<button
			className={cn(
				"flex w-full items-center justify-between gap-3 rounded-[1rem] border px-3 py-2.5 text-left text-sm transition",
				active
					? "border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]"
					: "border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
				dragActive &&
					"border-[var(--color-accent-gold-line)] shadow-[0_0_0_1px_var(--color-accent-gold-line)]",
			)}
			onClick={onClick}
			onContextMenu={onContextMenu}
			onDragLeave={onDragLeave}
			onDragOver={onDragOver}
			onDrop={onDrop}
			type="button"
		>
			<span className="flex min-w-0 items-center gap-2.5">
				<span className="text-[var(--color-text-muted)]">{icon}</span>
				<span className="truncate">{label}</span>
			</span>
			{typeof count === "number" ? (
				<Badge className="shrink-0 normal-case px-2.5 py-1" variant="subtle">
					{count}
				</Badge>
			) : null}
		</button>
	);
}

function truncateSummaryText(text: string, maxLength: number) {
	const normalizedText = normalizeSummaryText(text);
	const characters = Array.from(normalizedText);

	if (characters.length <= maxLength) {
		return normalizedText;
	}

	return `${characters.slice(0, maxLength).join("").trimEnd()}…`;
}

function demoCharacterNeedsCoverSync(
	definition: (typeof demoCharacterDefinitions)[number],
	summary?: CharacterSummary,
) {
	return Boolean(
		summary &&
		definition.coverUrl &&
		definition.coverFileName &&
		!summary.cover_file_name,
	);
}

function CharacterArtwork({
	coverUrl,
	mode,
	name,
}: {
	coverUrl?: string;
	mode: "card" | "dialog" | "list";
	name: string;
}) {
	const { t } = useTranslation();
	const monogram = getCharacterMonogram(name);
	const baseClasses =
		mode === "list"
			? "size-[4.25rem] rounded-full border border-[var(--color-border-subtle)] shadow-[0_12px_30px_rgba(0,0,0,0.18),inset_0_1px_0_rgba(255,255,255,0.08)]"
			: mode === "dialog"
				? "aspect-[4/3] rounded-[1.7rem] border border-[var(--color-border-subtle)]"
				: "aspect-[4/3] border-b border-[var(--color-border-subtle)]";

	return (
		<div
			className={cn(
				"overflow-hidden bg-[linear-gradient(135deg,var(--color-accent-gold-soft),var(--color-accent-copper-soft))]",
				baseClasses,
			)}
		>
			{coverUrl ? (
				<img
					alt={t("characters.card.coverAlt", { name })}
					className="h-full w-full object-cover transition duration-300 ease-out group-hover:scale-[1.02]"
					src={coverUrl}
					style={{ objectPosition: COVER_OBJECT_POSITION }}
				/>
			) : (
				<div className="flex h-full w-full items-center justify-center">
					<span
						className={cn(
							"inline-flex items-center justify-center rounded-full border border-[rgba(255,255,255,0.12)] bg-[rgba(18,10,31,0.34)] font-display text-[var(--color-text-primary)] shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]",
							mode === "list"
								? "size-11 text-lg"
								: mode === "dialog"
									? "size-24 text-4xl"
									: "size-16 text-[1.75rem]",
						)}
					>
						{monogram}
					</span>
				</div>
			)}
		</div>
	);
}

function ViewModeToggle({
	onChange,
	value,
}: {
	onChange: (value: CharacterViewMode) => void;
	value: CharacterViewMode;
}) {
	const { t } = useTranslation();
	const prefersReducedMotion = useReducedMotion();
	const items: Array<{
		icon: ReactNode;
		label: string;
		value: CharacterViewMode;
	}> = [
		{
			icon: <FontAwesomeIcon icon={faTableCellsLarge} />,
			label: t("characters.views.grid"),
			value: "grid",
		},
		{
			icon: <FontAwesomeIcon icon={faList} />,
			label: t("characters.views.list"),
			value: "list",
		},
	];

	return (
		<div
			aria-label={t("characters.views.label")}
			className="inline-flex items-center gap-1 rounded-[1.1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]"
			role="group"
		>
			{items.map((item) => {
				const selected = item.value === value;

				return (
					<button
						aria-label={item.label}
						aria-pressed={selected}
						className={cn(
							"relative inline-flex size-10 items-center justify-center rounded-[0.9rem] transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]",
							selected
								? "text-[color:var(--color-accent-ink)]"
								: "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
						)}
						key={item.value}
						onClick={() => {
							if (item.value !== value) {
								onChange(item.value);
							}
						}}
						title={item.label}
						type="button"
					>
						{selected ? (
							<motion.span
								className="absolute inset-0 rounded-[0.9rem] border border-[var(--color-accent-gold-line)] bg-[linear-gradient(135deg,var(--color-accent-gold),var(--color-accent-gold-strong))] shadow-[0_10px_28px_var(--color-accent-glow-soft)]"
								layoutId="characters-view-toggle-active"
								transition={
									prefersReducedMotion
										? { duration: 0 }
										: {
												damping: 34,
												mass: 0.34,
												stiffness: 420,
												type: "spring",
											}
								}
							/>
						) : null}
						<span className="relative z-10">
							{item.icon}
						</span>
					</button>
				);
			})}
		</div>
	);
}

function SelectionIndicator({ selected }: { selected: boolean }) {
	return (
		<span
			aria-hidden="true"
			className={cn(
				"inline-flex size-7 items-center justify-center rounded-full border text-xs shadow-[0_10px_24px_rgba(0,0,0,0.16)] transition",
				selected
					? "border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[color:var(--color-accent-ink)]"
					: "border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] text-[var(--color-text-muted)]",
			)}
		>
			{selected ? "✓" : ""}
		</span>
	);
}

function CharacterQuickActions({
	onDelete,
	onEdit,
}: {
	onDelete: () => void;
	onEdit: () => void;
}) {
	const { t } = useTranslation();

	return (
		<div className="flex items-center gap-1.5">
			<IconButton
				icon={<FontAwesomeIcon icon={faPen} />}
				label={t("characters.actions.edit")}
				onClick={onEdit}
				size="sm"
				variant="secondary"
			/>
			<IconButton
				icon={<FontAwesomeIcon icon={faTrashCan} />}
				label={t("characters.actions.delete")}
				onClick={onDelete}
				size="sm"
				variant="danger"
			/>
		</div>
	);
}

function LoadingGrid() {
	return (
		<div className="grid gap-4 xl:grid-cols-2 2xl:grid-cols-3">
			{Array.from({ length: 6 }).map((_, index) => (
				<div
					className="overflow-hidden rounded-[1.75rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] shadow-[0_24px_80px_rgba(0,0,0,0.18)]"
					key={index}
				>
					<div className="h-48 animate-pulse bg-[color-mix(in_srgb,var(--color-accent-gold-soft)_55%,var(--color-bg-elevated))]" />
					<div className="space-y-3 p-4">
						<div className="h-7 w-2/3 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
						<div className="h-3 w-28 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
						<div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
						<div className="h-3 w-5/6 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
					</div>
				</div>
			))}
		</div>
	);
}

function LoadingList() {
	return (
		<div className="space-y-3">
			{Array.from({ length: 6 }).map((_, index) => (
				<div
					className="overflow-hidden rounded-[1.75rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] shadow-[0_24px_80px_rgba(0,0,0,0.18)]"
					key={index}
				>
					<div className="grid gap-3.5 p-3 sm:grid-cols-[4.25rem_minmax(0,10.5rem)_minmax(0,1fr)] sm:items-center">
						<div className="size-[4.25rem] animate-pulse rounded-full border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-accent-gold-soft)_55%,var(--color-bg-elevated))]" />
						<div className="space-y-2.5">
							<div className="h-5 w-36 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
							<div className="h-3 w-28 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
						</div>
						<div className="hidden h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)] sm:block" />
						<div className="h-3 w-5/6 animate-pulse rounded-full bg-[var(--color-bg-elevated)] sm:hidden" />
					</div>
				</div>
			))}
		</div>
	);
}

function CharacterCard({
	coverUrl,
	draggable = false,
	onDragEnd,
	onDragStart,
	onDelete,
	onEdit,
	onOpenDetails,
	onToggleSelect,
	selected,
	selectionMode,
	summary,
}: {
	coverUrl?: string;
	draggable?: boolean;
	onDragEnd?: () => void;
	onDragStart?: (event: React.DragEvent<HTMLButtonElement>) => void;
	onDelete: () => void;
	onEdit: () => void;
	onOpenDetails: () => void;
	onToggleSelect: () => void;
	selected: boolean;
	selectionMode: boolean;
	summary: CharacterSummary;
}) {
	const { t } = useTranslation();
	const personalitySummary = truncateSummaryText(summary.personality, 72);

	return (
		<Card className="flex h-full flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)]">
			<button
				aria-pressed={selectionMode ? selected : undefined}
				className="group flex w-full flex-1 flex-col text-left"
				draggable={draggable}
				onDragEnd={onDragEnd}
				onDragStart={onDragStart}
				onClick={selectionMode ? onToggleSelect : onOpenDetails}
				type="button"
			>
				<CharacterArtwork coverUrl={coverUrl} mode="card" name={summary.name} />

				<CardHeader className="gap-1 p-4 pb-2">
					<div className="space-y-0.5">
						<CardTitle className="text-[1.48rem] leading-tight">
							{summary.name}
						</CardTitle>
						<CardDescription className="truncate font-mono text-[0.76rem] leading-[1.125rem] text-[var(--color-text-muted)]">
							{summary.character_id}
						</CardDescription>
					</div>
				</CardHeader>

				<CardContent className="flex-1 px-4 pb-3 pt-0">
					<div className="mb-3 flex flex-wrap items-center gap-2">
						<Badge className="normal-case px-2.5 py-1" variant="subtle">
							{summary.folder || t("characters.filters.unfiled")}
						</Badge>
						{summary.tags.slice(0, 3).map((tag) => (
							<Badge className="normal-case px-2.5 py-1" key={tag} variant="subtle">
								#{tag}
							</Badge>
						))}
					</div>
					<p
						className="text-sm leading-6 text-[var(--color-text-secondary)] transition group-hover:text-[var(--color-text-primary)]"
						style={CARD_EXCERPT_STYLE}
					>
						{personalitySummary}
					</p>
				</CardContent>
			</button>

			<div className="flex items-center justify-end px-4 pb-4 pt-0">
				{selectionMode ? (
					<SelectionIndicator selected={selected} />
				) : (
					<CharacterQuickActions onDelete={onDelete} onEdit={onEdit} />
				)}
			</div>
		</Card>
	);
}

function CharacterListRow({
	coverUrl,
	draggable = false,
	onDragEnd,
	onDragStart,
	onDelete,
	onEdit,
	onOpenDetails,
	onToggleSelect,
	selected,
	selectionMode,
	summary,
}: {
	coverUrl?: string;
	draggable?: boolean;
	onDragEnd?: () => void;
	onDragStart?: (event: React.DragEvent<HTMLButtonElement>) => void;
	onDelete: () => void;
	onEdit: () => void;
	onOpenDetails: () => void;
	onToggleSelect: () => void;
	selected: boolean;
	selectionMode: boolean;
	summary: CharacterSummary;
}) {
	const { t } = useTranslation();
	const personalitySummary = normalizeSummaryText(summary.personality);

	return (
		<Card className="overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)]">
			<div className="grid gap-3.5 p-3 sm:grid-cols-[4.25rem_minmax(0,10.5rem)_minmax(0,1fr)_auto] sm:items-center">
				<button
					aria-pressed={selectionMode ? selected : undefined}
					className="group grid min-w-0 gap-3.5 text-left sm:col-span-3 sm:grid-cols-[4.25rem_minmax(0,10.5rem)_minmax(0,1fr)] sm:items-center"
					draggable={draggable}
					onDragEnd={onDragEnd}
					onDragStart={onDragStart}
					onClick={selectionMode ? onToggleSelect : onOpenDetails}
					type="button"
				>
					<div className="relative flex items-center justify-center">
						<CharacterArtwork
							coverUrl={coverUrl}
							mode="list"
							name={summary.name}
						/>
						{selectionMode ? (
							<span className="absolute -right-1 -top-1 sm:hidden">
								<SelectionIndicator selected={selected} />
							</span>
						) : null}
					</div>

					<div className="min-w-0 space-y-0.5">
						<CardTitle className="truncate text-[1.08rem] leading-tight">
							{summary.name}
						</CardTitle>
						<CardDescription className="truncate font-mono text-[0.72rem] leading-5 text-[var(--color-text-muted)]">
							{summary.character_id}
						</CardDescription>
					</div>

					<div className="min-w-0 pl-1 sm:pl-2">
						<div className="mb-1.5 flex flex-wrap items-center gap-2">
							<Badge className="normal-case px-2.5 py-1" variant="subtle">
								{summary.folder || t("characters.filters.unfiled")}
							</Badge>
							{summary.tags.slice(0, 2).map((tag) => (
								<Badge className="normal-case px-2.5 py-1" key={tag} variant="subtle">
									#{tag}
								</Badge>
							))}
						</div>
						<p
							className="text-[0.92rem] leading-5 text-[var(--color-text-secondary)] transition group-hover:text-[var(--color-text-primary)]"
							style={LIST_EXCERPT_STYLE}
						>
							{personalitySummary}
						</p>
					</div>
				</button>

				{selectionMode ? (
					<div className="hidden sm:flex sm:justify-end">
						<SelectionIndicator selected={selected} />
					</div>
				) : (
					<div className="flex justify-start sm:justify-end">
						<CharacterQuickActions onDelete={onDelete} onEdit={onEdit} />
					</div>
				)}
				</div>
		</Card>
	);
}

function CharacterResults({
	characters,
	coverUrls,
	isLoading,
	onDelete,
	onDragEnd,
	onDragStart,
	onEdit,
	onOpenDetails,
	onToggleSelect,
	selectedCharacterIds,
	selectionMode,
	viewMode,
}: {
	characters: CharacterSummary[];
	coverUrls: Record<string, string>;
	isLoading: boolean;
	onDelete: (characterId: string) => void;
	onDragEnd: () => void;
	onDragStart: (characterId: string, event: React.DragEvent<HTMLButtonElement>) => void;
	onEdit: (characterId: string) => void;
	onOpenDetails: (characterId: string) => void;
	onToggleSelect: (characterId: string) => void;
	selectedCharacterIds: ReadonlySet<string>;
	selectionMode: boolean;
	viewMode: CharacterViewMode;
}) {
	const prefersReducedMotion = useReducedMotion();

	let content: ReactNode;
	let contentKey: string = viewMode;

	if (isLoading) {
		content = viewMode === "list" ? <LoadingList /> : <LoadingGrid />;
		contentKey = `loading-${viewMode}`;
	} else if (viewMode === "list") {
		content = (
			<div className="space-y-3">
				{characters.map((summary) => (
					<div key={summary.character_id}>
						<CharacterListRow
							coverUrl={coverUrls[summary.character_id]}
							draggable={!selectionMode}
							onDragEnd={onDragEnd}
							onDragStart={(event) => {
								onDragStart(summary.character_id, event);
							}}
							onDelete={() => {
								onDelete(summary.character_id);
							}}
							onEdit={() => {
								onEdit(summary.character_id);
							}}
							onOpenDetails={() => {
								onOpenDetails(summary.character_id);
							}}
							onToggleSelect={() => {
								onToggleSelect(summary.character_id);
							}}
							selected={selectedCharacterIds.has(summary.character_id)}
							selectionMode={selectionMode}
							summary={summary}
						/>
					</div>
				))}
			</div>
		);
	} else {
		content = (
			<div className="grid gap-4 xl:grid-cols-2 2xl:grid-cols-3">
				{characters.map((summary) => (
					<div key={summary.character_id}>
						<CharacterCard
							coverUrl={coverUrls[summary.character_id]}
							draggable={!selectionMode}
							onDragEnd={onDragEnd}
							onDragStart={(event) => {
								onDragStart(summary.character_id, event);
							}}
							onDelete={() => {
								onDelete(summary.character_id);
							}}
							onEdit={() => {
								onEdit(summary.character_id);
							}}
							onOpenDetails={() => {
								onOpenDetails(summary.character_id);
							}}
							onToggleSelect={() => {
								onToggleSelect(summary.character_id);
							}}
							selected={selectedCharacterIds.has(summary.character_id)}
							selectionMode={selectionMode}
							summary={summary}
						/>
					</div>
				))}
			</div>
		);
	}

	return (
		<AnimatePresence initial={false} mode="wait">
			<motion.div
				animate={prefersReducedMotion ? undefined : { opacity: 1, y: 0 }}
				exit={prefersReducedMotion ? undefined : { opacity: 0, y: 8 }}
				initial={prefersReducedMotion ? undefined : { opacity: 0, y: -8 }}
				key={contentKey}
				transition={
					prefersReducedMotion
						? { duration: 0 }
						: { duration: 0.24, ease: [0.22, 1, 0.36, 1] }
				}
			>
				{content}
			</motion.div>
		</AnimatePresence>
	);
}

export function CharacterManagementPage() {
	const { t } = useTranslation();
	const { setRailContent } = useWorkspaceLayoutContext();
	const importInputRef = useRef<HTMLInputElement | null>(null);
	const coverCacheRef = useRef<Map<string, string>>(new Map());
	const [characters, setCharacters] = useState<CharacterSummary[]>([]);
	const [folderRegistry, setFolderRegistry] = useState<string[]>(() =>
		loadCharacterFolderRegistry(),
	);
	const [coverUrls, setCoverUrls] = useState<Record<string, string>>({});
	const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([]);
	const [editingCharacterId, setEditingCharacterId] = useState<string | null>(
		null,
	);
	const [exportingCharacterId, setExportingCharacterId] = useState<
		string | null
	>(null);
	const [isDeleting, setIsDeleting] = useState(false);
	const [isCharacterFormOpen, setIsCharacterFormOpen] = useState(false);
	const [isCreatingSamples, setIsCreatingSamples] = useState(false);
	const [isSampleDialogOpen, setIsSampleDialogOpen] = useState(false);
	const [isImporting, setIsImporting] = useState(false);
	const [isLoading, setIsLoading] = useState(true);
	const [notice, setNotice] = useState<Notice | null>(null);
	useToastNotice(notice);
	const [selectedCharacterId, setSelectedCharacterId] = useState<string | null>(
		null,
	);
	const [selectedCharacterIds, setSelectedCharacterIds] = useState<string[]>([]);
	const [selectionMode, setSelectionMode] = useState(false);
	const [viewMode, setViewMode] = useState<CharacterViewMode>("grid");
	const [searchQuery, setSearchQuery] = useState("");
	const [activeFolder, setActiveFolder] = useState<string>("all");
	const [draggedCharacterId, setDraggedCharacterId] = useState<string | null>(null);
	const [folderDropTarget, setFolderDropTarget] = useState<string | "__unfiled__" | null>(
		null,
	);
	const [folderDialogState, setFolderDialogState] = useState<FolderDialogState | null>(
		null,
	);
	const [folderDialogValue, setFolderDialogValue] = useState("");
	const [folderDeleteTarget, setFolderDeleteTarget] = useState<string | null>(null);
	const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
	const [isApplyingFolderChange, setIsApplyingFolderChange] = useState(false);
	const [isDeletingFolder, setIsDeletingFolder] = useState(false);
	const [isExplorerOpen, setIsExplorerOpen] = useState(false);
	const prefersReducedMotion = useReducedMotion();

	const selectedCharacter =
		selectedCharacterId !== null
			? (characters.find(
					(character) => character.character_id === selectedCharacterId,
				) ?? null)
			: null;
	const selectedCharacterSet = useMemo(
		() => new Set(selectedCharacterIds),
		[selectedCharacterIds],
	);
	const availableFolders = useMemo(
		() =>
			Array.from(
				new Set([
					...folderRegistry,
					...characters.map((character) => normalizeCharacterFolder(character.folder)),
				]),
			).sort((left, right) => left.localeCompare(right)),
		[characters, folderRegistry],
	);
	const filteredCharacters = useMemo(() => {
		const normalizedQuery = searchQuery.trim().toLocaleLowerCase();

		return characters.filter((character) => {
			const folder = normalizeCharacterFolder(character.folder);
			const matchesFolder =
				activeFolder === "all" ||
				(activeFolder === "__unfiled__" ? folder.length === 0 : folder === activeFolder);
			const matchesSearch =
				normalizedQuery.length === 0 ||
				buildCharacterSearchText(character).includes(normalizedQuery);

			return matchesFolder && matchesSearch;
		});
	}, [activeFolder, characters, searchQuery]);
	const deleteTargets = useMemo(
		() =>
			deleteTargetIds
				.map((characterId) =>
					characters.find(
						(character) => character.character_id === characterId,
					),
				)
				.filter((character): character is CharacterSummary => character !== undefined),
		[characters, deleteTargetIds],
	);
	const folderDeleteCharacters = useMemo(
		() =>
			folderDeleteTarget === null
				? []
				: characters.filter(
						(character) =>
							normalizeCharacterFolder(character.folder) === folderDeleteTarget,
					),
		[characters, folderDeleteTarget],
	);

	const refreshCharacters = useCallback(
		async (signal?: AbortSignal) => {
			setIsLoading(true);

			try {
				const summaries = await listCharacters(signal);

				if (signal?.aborted) {
					return;
				}

				const availableIds = new Set(
					summaries.map((summary) => summary.character_id),
				);

				for (const [characterId, coverUrl] of coverCacheRef.current.entries()) {
					if (!availableIds.has(characterId)) {
						revokeObjectUrl(coverUrl);
						coverCacheRef.current.delete(characterId);
					}
				}

				setCharacters(summaries);

				const cachedCoverUrls: Record<string, string> = {};

				for (const summary of summaries) {
					const cachedCoverUrl = coverCacheRef.current.get(
						summary.character_id,
					);

					if (cachedCoverUrl) {
						cachedCoverUrls[summary.character_id] = cachedCoverUrl;
					}
				}

				setCoverUrls(cachedCoverUrls);

				const summariesNeedingCover = summaries.filter(
					(summary) =>
						summary.cover_file_name &&
						summary.cover_mime_type &&
						!coverCacheRef.current.has(summary.character_id),
				);

				if (summariesNeedingCover.length === 0) {
					return;
				}

				const coverResults = await Promise.allSettled(
					summariesNeedingCover.map(async (summary) => {
						return {
							characterId: summary.character_id,
							coverUrl: await getCharacterCoverUrl(summary.character_id, signal),
						};
					}),
				);

				if (signal?.aborted) {
					for (const result of coverResults) {
						if (result.status === "fulfilled") {
							revokeObjectUrl(result.value.coverUrl);
						}
					}

					return;
				}

				const nextCoverUrls: Record<string, string> = {};

				for (const result of coverResults) {
					if (result.status !== "fulfilled") {
						continue;
					}

					coverCacheRef.current.set(
						result.value.characterId,
						result.value.coverUrl,
					);
					nextCoverUrls[result.value.characterId] = result.value.coverUrl;
				}

				if (Object.keys(nextCoverUrls).length > 0) {
					setCoverUrls((currentCoverUrls) => ({
						...currentCoverUrls,
						...nextCoverUrls,
					}));
				}
			} catch (error) {
				if (signal?.aborted) {
					return;
				}

				setNotice({
					message: getErrorMessage(error, t("characters.feedback.loadFailed")),
					tone: "error",
				});
			} finally {
				if (!signal?.aborted) {
					setIsLoading(false);
				}
			}
		},
		[t],
	);

	useEffect(() => {
		const controller = new AbortController();

		void refreshCharacters(controller.signal);

		return () => {
			controller.abort();
		};
	}, [refreshCharacters]);

	useEffect(() => {
		const coverCache = coverCacheRef.current;

		return () => {
			for (const coverUrl of coverCache.values()) {
				revokeObjectUrl(coverUrl);
			}

			coverCache.clear();
		};
	}, []);

	useEffect(() => {
		saveCharacterFolderRegistry(folderRegistry);
	}, [folderRegistry]);

	useEffect(() => {
		const foldersFromCharacters = Array.from(
			new Set(
				characters
					.map((character) => normalizeCharacterFolder(character.folder))
					.filter((folder) => folder.length > 0),
			),
		).sort((left, right) => left.localeCompare(right));

		setFolderRegistry((current) => {
			const merged = Array.from(
				new Set([...current, ...foldersFromCharacters]),
			).sort((left, right) => left.localeCompare(right));

			if (
				merged.length === current.length &&
				merged.every((folder, index) => folder === current[index])
			) {
				return current;
			}

			return merged;
		});
	}, [characters]);

	useEffect(() => {
		if (!isExplorerOpen) {
			setContextMenu(null);
		}
	}, [isExplorerOpen]);

	useEffect(() => {
		const availableIds = new Set(characters.map((character) => character.character_id));

		setSelectedCharacterIds((currentSelection) =>
			currentSelection.filter((characterId) => availableIds.has(characterId)),
		);
		setDeleteTargetIds((currentSelection) =>
			currentSelection.filter((characterId) => availableIds.has(characterId)),
		);

		if (
			selectedCharacterId !== null &&
			!availableIds.has(selectedCharacterId)
		) {
			setSelectedCharacterId(null);
		}

		if (
			editingCharacterId !== null &&
			!availableIds.has(editingCharacterId)
		) {
			setEditingCharacterId(null);
		}
	}, [characters, editingCharacterId, selectedCharacterId]);

	useLayoutEffect(() => {
		setRailContent({
			description: t("characters.rail.description"),
			stats: [
				{
					label: t("characters.metrics.total"),
					value: filteredCharacters.length,
				},
			],
			title: t("characters.title"),
		});

		return () => {
			setRailContent(null);
		};
	}, [filteredCharacters.length, setRailContent, t]);

	function clearCoverEntries(characterIds: ReadonlyArray<string>) {
		if (characterIds.length === 0) {
			return;
		}

		for (const characterId of characterIds) {
			revokeObjectUrl(coverCacheRef.current.get(characterId));
			coverCacheRef.current.delete(characterId);
		}

		setCoverUrls((currentCoverUrls) =>
			Object.fromEntries(
				Object.entries(currentCoverUrls).filter(
					([characterId]) => !characterIds.includes(characterId),
				),
			),
		);
	}

	function openCreateDialog() {
		setEditingCharacterId(null);
		setIsCharacterFormOpen(true);
	}

	function openEditDialog(characterId: string) {
		setSelectedCharacterId(null);
		setEditingCharacterId(characterId);
		setIsCharacterFormOpen(true);
	}

	function openFolderCreateDialog() {
		setContextMenu(null);
		setFolderDialogState({ mode: "create" });
		setFolderDialogValue("");
	}

	function openFolderRenameDialog(folder: string) {
		setContextMenu(null);
		setFolderDialogState({
			folder,
			mode: "rename",
		});
		setFolderDialogValue(folder);
	}

	function openFolderDeleteDialog(folder: string) {
		setContextMenu(null);
		setFolderDeleteTarget(folder);
	}

	async function moveCharactersToFolder(
		characterIds: string[],
		folder: string,
		options?: {
			silent?: boolean;
		},
	) {
		if (characterIds.length === 0) {
			return;
		}

		const normalizedFolder = normalizeCharacterFolderRegistryName(folder);
		setIsApplyingFolderChange(true);

		try {
			for (const characterId of characterIds) {
				const detail = await getCharacter(characterId);
				await updateCharacter({
					characterId,
					content: {
						...detail.content,
						folder: normalizedFolder,
					},
				});
			}

			if (!options?.silent) {
				setNotice({
					message: normalizedFolder
						? t("characters.feedback.movedToFolder", {
								count: characterIds.length,
								folder: normalizedFolder,
							})
						: t("characters.feedback.movedToUnfiled", {
								count: characterIds.length,
							}),
					tone: "success",
				});
			}

			await refreshCharacters();
		} catch (error) {
			setNotice({
				message: getErrorMessage(error, t("characters.feedback.folderActionFailed")),
				tone: "error",
			});
		} finally {
			setDraggedCharacterId(null);
			setFolderDropTarget(null);
			setIsApplyingFolderChange(false);
		}
	}

	async function handleFolderDialogSubmit() {
		if (!folderDialogState) {
			return;
		}

		const normalizedFolder = normalizeCharacterFolderRegistryName(folderDialogValue);
		if (!normalizedFolder) {
			setNotice({
				message: t("characters.feedback.folderNameRequired"),
				tone: "warning",
			});
			return;
		}

		if (
			folderDialogState.mode === "create" &&
			availableFolders.includes(normalizedFolder)
		) {
			setNotice({
				message: t("characters.feedback.folderExists", {
					folder: normalizedFolder,
				}),
				tone: "warning",
			});
			return;
		}

		if (
			folderDialogState.mode === "rename" &&
			normalizedFolder !== folderDialogState.folder &&
			availableFolders.includes(normalizedFolder)
		) {
			setNotice({
				message: t("characters.feedback.folderExists", {
					folder: normalizedFolder,
				}),
				tone: "warning",
			});
			return;
		}

		if (folderDialogState.mode === "create") {
			setFolderRegistry((current) =>
				addCharacterFolderRegistryEntry(current, normalizedFolder),
			);
			setFolderDialogState(null);
			setFolderDialogValue("");
			setActiveFolder(normalizedFolder);
			setNotice({
				message: t("characters.feedback.folderCreated", {
					folder: normalizedFolder,
				}),
				tone: "success",
			});
			return;
		}

		const currentFolder = folderDialogState.folder;
		setFolderRegistry((current) =>
			renameCharacterFolderRegistryEntry(current, currentFolder, normalizedFolder),
		);
		setFolderDialogState(null);
		setFolderDialogValue("");
		if (activeFolder === currentFolder) {
			setActiveFolder(normalizedFolder);
		}

		const affectedCharacters = characters
			.filter((character) => normalizeCharacterFolder(character.folder) === currentFolder)
			.map((character) => character.character_id);

		await moveCharactersToFolder(affectedCharacters, normalizedFolder, {
			silent: true,
		});
		setNotice({
			message: t("characters.feedback.folderRenamed", {
				from: currentFolder,
				to: normalizedFolder,
			}),
			tone: "success",
		});
	}

	async function handleFolderDelete() {
		if (folderDeleteTarget === null) {
			return;
		}

		const targetFolder = folderDeleteTarget;
		const targets = folderDeleteCharacters;
		setIsDeletingFolder(true);

		const deletedIds: string[] = [];
		let firstDeleteError: unknown = null;

		try {
			for (const target of targets) {
				try {
					await deleteCharacter(target.character_id);
					deletedIds.push(target.character_id);
				} catch (error) {
					if (firstDeleteError === null) {
						firstDeleteError = error;
					}
				}
			}

			if (deletedIds.length > 0) {
				clearCoverEntries(deletedIds);
				setSelectedCharacterIds((currentSelection) =>
					currentSelection.filter(
						(characterId) => !deletedIds.includes(characterId),
					),
				);

				setDeleteTargetIds((currentSelection) =>
					currentSelection.filter(
						(characterId) => !deletedIds.includes(characterId),
					),
				);

				if (
					selectedCharacterId !== null &&
					deletedIds.includes(selectedCharacterId)
				) {
					setSelectedCharacterId(null);
				}
			}

			if (targets.length > 0 && deletedIds.length !== targets.length) {
				setNotice({
					message: getErrorMessage(
						firstDeleteError,
						t("characters.feedback.folderDeleteFailed", {
							folder: targetFolder,
						}),
					),
					tone: "error",
				});
				return;
			}

			setFolderRegistry((current) =>
				removeCharacterFolderRegistryEntry(current, targetFolder),
			);

			if (activeFolder === targetFolder) {
				setActiveFolder("all");
			}

			setNotice({
				message: t("characters.feedback.folderDeleted", {
					folder: targetFolder,
				}),
				tone: "success",
			});
			setFolderDeleteTarget(null);
			await refreshCharacters();
		} catch (error) {
			setNotice({
				message: getErrorMessage(
					error,
					t("characters.feedback.folderDeleteFailed", {
						folder: targetFolder,
					}),
				),
				tone: "error",
			});
		} finally {
			setIsDeletingFolder(false);
		}
	}

	async function ensureActorSchemaId() {
		const actorPreset = buildSchemaPresetDefinitions(t).find(
			(preset) => preset.kind === "actor",
		);

		if (!actorPreset) {
			throw new Error(t("characters.feedback.demoSchemaMissing"));
		}

		const existingSchemas = await listSchemas();

		if (existingSchemas.some((schema) => schema.schema_id === actorPreset.schemaId)) {
			return actorPreset.schemaId;
		}

		try {
			await createSchema({
				display_name: actorPreset.displayName,
				fields: actorPreset.fields,
				schema_id: actorPreset.schemaId,
				tags: actorPreset.tags,
			});
		} catch (error) {
			if (!isRpcConflict(error)) {
				throw error;
			}
		}

		return actorPreset.schemaId;
	}

	async function handleCreateDemoCharacters() {
		const existingCharacters = new Map(
			characters.map((character) => [character.character_id, character]),
		);
		const hasPendingSampleChanges = demoCharacterDefinitions.some((definition) => {
			const existingCharacter = existingCharacters.get(definition.characterId);

			if (!existingCharacter) {
				return true;
			}

			return demoCharacterNeedsCoverSync(definition, existingCharacter);
		});

		if (!hasPendingSampleChanges) {
			setNotice({
				message: t("characters.feedback.samplesExist"),
				tone: "warning",
			});
			return;
		}

		setIsCreatingSamples(true);

		const createdNames: string[] = [];
		const failedNames: string[] = [];
		const processedNames: string[] = [];
		const skippedNames: string[] = [];

		try {
			const schemaId = await ensureActorSchemaId();

			for (const definition of demoCharacterDefinitions) {
				const existingCharacter = existingCharacters.get(definition.characterId);

				if (existingCharacter) {
					if (demoCharacterNeedsCoverSync(definition, existingCharacter)) {
						try {
							const coverFile = await loadDemoCoverFile(
								definition.coverUrl!,
								definition.coverFileName!,
							);

							await setCharacterCover({
								characterId: existingCharacter.character_id,
								coverFile,
							});

							processedNames.push(definition.content.name);
						} catch {
							failedNames.push(definition.content.name);
						}

						continue;
					}

					skippedNames.push(definition.content.name);
					continue;
				}

				try {
					const created = await createCharacter({
						...definition.content,
						schema_id: schemaId,
					});
					if (definition.coverUrl && definition.coverFileName) {
						const coverFile = await loadDemoCoverFile(
							definition.coverUrl,
							definition.coverFileName,
						);

						await setCharacterCover({
							characterId: created.character_id,
							coverFile,
						});
					}

					existingCharacters.set(created.character_id, created.character_summary);
					createdNames.push(definition.content.name);
					processedNames.push(definition.content.name);
				} catch {
					failedNames.push(definition.content.name);
				}
			}

			if (processedNames.length > 0) {
				await refreshCharacters();
			}

			if (failedNames.length === 0 && processedNames.length > 0 && skippedNames.length === 0) {
				setNotice({
					message: t("characters.feedback.samplesCreated", {
						names: processedNames.join("、"),
					}),
					tone: "success",
				});
			} else if (processedNames.length > 0 && (skippedNames.length > 0 || failedNames.length > 0)) {
				setNotice({
					message: t("characters.feedback.samplesCreatedPartial", {
						created: processedNames.join("、"),
						skipped: [...skippedNames, ...failedNames].join("、"),
					}),
					tone: "warning",
				});
			} else {
				setNotice({
					message:
						skippedNames.length > 0
							? t("characters.feedback.samplesExist")
							: t("characters.feedback.sampleCreateFailed"),
					tone: skippedNames.length > 0 ? "warning" : "error",
				});
			}
		} catch (error) {
			setNotice({
				message: getErrorMessage(error, t("characters.feedback.sampleCreateFailed")),
				tone: "error",
			});
		} finally {
			setIsCreatingSamples(false);
		}
	}

	function exitSelectionMode() {
		setSelectionMode(false);
		setSelectedCharacterIds([]);
	}

	function toggleCharacterSelection(characterId: string) {
		setSelectedCharacterIds((currentSelection) =>
			currentSelection.includes(characterId)
				? currentSelection.filter((currentId) => currentId !== characterId)
				: [...currentSelection, characterId],
		);
	}

	function requestDelete(characterIds: string[]) {
		if (characterIds.length === 0) {
			return;
		}

		setDeleteTargetIds(characterIds);
	}

	function openRootContextMenu(event: React.MouseEvent<HTMLElement>) {
		event.preventDefault();
		event.stopPropagation();
		setContextMenu({
			target: "root",
			x: event.clientX,
			y: event.clientY,
		});
	}

	function openFolderContextMenu(
		event: React.MouseEvent<HTMLElement>,
		folder: string,
	) {
		event.preventDefault();
		event.stopPropagation();
		setContextMenu({
			folder,
			target: "folder",
			x: event.clientX,
			y: event.clientY,
		});
	}

	function handleCharacterDragStart(
		characterId: string,
		event: React.DragEvent<HTMLButtonElement>,
	) {
		event.dataTransfer.effectAllowed = "move";
		event.dataTransfer.setData("text/plain", characterId);
		setDraggedCharacterId(characterId);
	}

	function handleCharacterDragEnd() {
		setDraggedCharacterId(null);
		setFolderDropTarget(null);
	}

	function handleFolderDragOver(
		event: React.DragEvent<HTMLElement>,
		folder: string | "__unfiled__",
	) {
		if (!draggedCharacterId) {
			return;
		}

		event.preventDefault();
		if (folderDropTarget !== folder) {
			setFolderDropTarget(folder);
		}
	}

	function handleFolderDragLeave(folder: string | "__unfiled__") {
		if (folderDropTarget === folder) {
			setFolderDropTarget(null);
		}
	}

	function handleFolderDrop(
		event: React.DragEvent<HTMLElement>,
		folder: string | "__unfiled__",
	) {
		if (!draggedCharacterId) {
			return;
		}

		event.preventDefault();
		void moveCharactersToFolder(
			[draggedCharacterId],
			folder === "__unfiled__" ? "" : folder,
		);
	}

	async function handleImportSelection(event: ChangeEvent<HTMLInputElement>) {
		const selectedFile = event.target.files?.[0];

		event.target.value = "";

		if (!selectedFile) {
			return;
		}

		if (!hasCharacterCardExtension(selectedFile.name)) {
			setNotice({
				message: t("characters.feedback.invalidImportType"),
				tone: "error",
			});
			return;
		}

		setIsImporting(true);

		try {
			const importedCharacter = await importCharacterArchive(selectedFile);

			setNotice({
				message: t("characters.feedback.imported", {
					name: importedCharacter.name,
				}),
				tone: "success",
			});

			await refreshCharacters();
		} catch (error) {
			setNotice({
				message: getErrorMessage(error, t("characters.feedback.importFailed")),
				tone: "error",
			});
		} finally {
			setIsImporting(false);
		}
	}

	async function handleExport(summary: CharacterSummary) {
		setExportingCharacterId(summary.character_id);

		try {
			await downloadCharacterArchive(summary.character_id);

			setNotice({
				message: t("characters.feedback.exported", { name: summary.name }),
				tone: "success",
			});
		} catch (error) {
			setNotice({
				message: isRpcConflict(error)
					? t("characters.feedback.exportNeedsCover", { name: summary.name })
					: getErrorMessage(error, t("characters.feedback.exportFailed")),
				tone: isRpcConflict(error) ? "warning" : "error",
			});
		} finally {
			setExportingCharacterId(null);
		}
	}

	async function handleDeleteCharacters() {
		if (deleteTargets.length === 0) {
			return;
		}

		setIsDeleting(true);

		const deletedIds: string[] = [];
		const failedTargets: CharacterSummary[] = [];
		let firstDeleteError: unknown = null;

		try {
			for (const target of deleteTargets) {
				try {
					await deleteCharacter(target.character_id);
					deletedIds.push(target.character_id);
				} catch (error) {
					failedTargets.push(target);
					if (firstDeleteError === null) {
						firstDeleteError = error;
					}
				}
			}

			clearCoverEntries(deletedIds);
			setDeleteTargetIds([]);

			if (deletedIds.length > 0) {
				setSelectedCharacterIds((currentSelection) =>
					currentSelection.filter(
						(characterId) => !deletedIds.includes(characterId),
					),
				);

				if (
					selectedCharacterId !== null &&
					deletedIds.includes(selectedCharacterId)
				) {
					setSelectedCharacterId(null);
				}
			}

			if (failedTargets.length === 0) {
				setNotice({
					message:
						deletedIds.length > 1
							? t("characters.feedback.deletedMany", {
								count: deletedIds.length,
							})
							: t("characters.feedback.deleted", {
								name: deleteTargets[0]?.name ?? "",
							}),
					tone: "success",
				});
			} else if (deletedIds.length > 0) {
				setNotice({
					message: t("characters.feedback.deletedPartial", {
						failed: failedTargets.length,
						success: deletedIds.length,
					}),
					tone: "warning",
				});
			} else {
				setNotice({
					message: getErrorMessage(
						firstDeleteError,
						t("characters.feedback.deleteFailed"),
					),
					tone: "error",
				});
			}

			if (selectionMode && deletedIds.length > 0 && failedTargets.length === 0) {
				exitSelectionMode();
			}

			await refreshCharacters();
		} finally {
			setIsDeleting(false);
		}
	}

	return (
		<div className="flex h-full min-h-0 flex-col gap-6">
			<CharacterDetailsDialog
				coverUrl={
					selectedCharacter
						? coverUrls[selectedCharacter.character_id]
						: undefined
				}
				exporting={
					selectedCharacter !== null &&
					exportingCharacterId === selectedCharacter.character_id
				}
				deleting={
					isDeleting &&
					selectedCharacter !== null &&
					deleteTargetIds.includes(selectedCharacter.character_id)
				}
				onDelete={() => {
					if (!selectedCharacter) {
						return;
					}

					requestDelete([selectedCharacter.character_id]);
				}}
				onEdit={() => {
					if (!selectedCharacter) {
						return;
					}

					openEditDialog(selectedCharacter.character_id);
				}}
				onExport={() => {
					if (!selectedCharacter) {
						return;
					}

					void handleExport(selectedCharacter);
				}}
				onOpenChange={(open) => {
					if (!open) {
						setSelectedCharacterId(null);
					}
				}}
				open={selectedCharacter !== null}
				summary={selectedCharacter}
			/>

			<CharacterFormDialog
				characterId={editingCharacterId}
				mode={editingCharacterId === null ? "create" : "edit"}
				onCompleted={async (result) => {
					setNotice({
						message: result.message,
						tone: result.tone,
					});

					if (result.coverUpdated) {
						clearCoverEntries([result.characterId]);
					}

					await refreshCharacters();
				}}
				onOpenChange={(open) => {
					setIsCharacterFormOpen(open);

					if (!open) {
						setEditingCharacterId(null);
					}
				}}
				open={isCharacterFormOpen}
			/>

			<InsertSampleDialog
				cancelLabel={t("characters.actions.cancel")}
				confirmLabel={t("characters.sampleDialog.confirm")}
				confirmDisabled={!demoCharacterDefinitions.some((definition) => {
					const existingCharacter = characters.find(
						(character) => character.character_id === definition.characterId,
					);

					if (!existingCharacter) {
						return true;
					}

					return demoCharacterNeedsCoverSync(definition, existingCharacter);
				})}
				description={t("characters.sampleDialog.description")}
				existingLabel={t("characters.sampleDialog.existing")}
				items={demoCharacterDefinitions.map((definition) => ({
					description: definition.characterId,
					label: definition.content.name,
					status: characters.some(
						(character) => character.character_id === definition.characterId,
					)
						? "existing"
						: "new",
				}))}
				newLabel={t("characters.sampleDialog.new")}
				onConfirm={() => {
					void handleCreateDemoCharacters();
					setIsSampleDialogOpen(false);
				}}
				onOpenChange={setIsSampleDialogOpen}
				open={isSampleDialogOpen}
				pending={isCreatingSamples}
				pendingLabel={t("characters.actions.creatingSamples")}
				title={t("characters.sampleDialog.title")}
			/>

			<DeleteCharacterDialog
				deleting={isDeleting}
				onConfirm={() => {
					void handleDeleteCharacters();
				}}
				onOpenChange={() => {
					setDeleteTargetIds([]);
				}}
				targets={deleteTargets}
			/>

			<Dialog
				onOpenChange={(open) => {
					if (!open) {
						setFolderDeleteTarget(null);
					}
				}}
				open={folderDeleteTarget !== null}
			>
				<DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
					<DialogHeader className="border-b border-[var(--color-border-subtle)]">
						<DialogTitle>{t("characters.folderDeleteDialog.title")}</DialogTitle>
					</DialogHeader>

					<DialogBody className="space-y-5 pt-6">
						<p className="text-sm leading-7 text-[var(--color-text-secondary)]">
							{t("characters.folderDeleteDialog.message", {
								count: folderDeleteCharacters.length,
								folder: folderDeleteTarget ?? "",
							})}
						</p>

						{folderDeleteCharacters.length > 0 ? (
							<div className="flex flex-wrap gap-2">
								{folderDeleteCharacters.slice(0, 5).map((target) => (
									<Badge
										className="normal-case px-3 py-1.5"
										key={target.character_id}
										variant="subtle"
									>
										{target.name}
									</Badge>
								))}
								{folderDeleteCharacters.length > 5 ? (
									<Badge className="normal-case px-3 py-1.5" variant="subtle">
										{t("characters.folderDeleteDialog.more", {
											count: folderDeleteCharacters.length - 5,
										})}
									</Badge>
								) : null}
							</div>
						) : null}
					</DialogBody>

					<DialogFooter>
						<DialogClose asChild>
							<Button disabled={isDeletingFolder} variant="ghost">
								{t("characters.actions.cancel")}
							</Button>
						</DialogClose>

						<Button
							disabled={isDeletingFolder}
							onClick={() => {
								void handleFolderDelete();
							}}
							variant="danger"
						>
							{isDeletingFolder
								? t("characters.actions.deleting")
								: t("characters.folders.menuDelete")}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>

			<Dialog
				onOpenChange={(open) => {
					if (!open) {
						setFolderDialogState(null);
						setFolderDialogValue("");
					}
				}}
				open={folderDialogState !== null}
			>
				<DialogContent aria-describedby={undefined} className="max-w-xl">
					<DialogHeader className="border-b border-[var(--color-border-subtle)]">
						<DialogTitle>
							{folderDialogState?.mode === "rename"
								? t("characters.folders.renameTitle")
								: t("characters.folders.createTitle")}
						</DialogTitle>
					</DialogHeader>
					<DialogBody className="pt-6">
						<div className="space-y-3">
							<label
								className="text-sm font-medium text-[var(--color-text-primary)]"
								htmlFor="character-folder-name"
							>
								{t("characters.folders.nameLabel")}
							</label>
							<Input
								autoFocus
								id="character-folder-name"
								onChange={(event) => {
									setFolderDialogValue(event.target.value);
								}}
								placeholder={t("characters.folders.namePlaceholder")}
								value={folderDialogValue}
							/>
						</div>
					</DialogBody>
					<DialogFooter>
						<DialogClose asChild>
							<Button size="md" variant="ghost">
								{t("characters.actions.cancel")}
							</Button>
						</DialogClose>
						<Button
							disabled={isApplyingFolderChange}
							onClick={() => {
								void handleFolderDialogSubmit();
							}}
							size="md"
						>
							{folderDialogState?.mode === "rename"
								? t("characters.folders.renameAction")
								: t("characters.folders.createAction")}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>

			{contextMenu && isExplorerOpen ? (
				<div
					className="fixed inset-0 z-50"
					onClick={() => {
						setContextMenu(null);
					}}
				>
					<div
						className="absolute w-52 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-2 shadow-[var(--shadow-floating)] backdrop-blur-xl"
						onClick={(event) => {
							event.stopPropagation();
						}}
						onContextMenu={(event) => {
							event.preventDefault();
							event.stopPropagation();
						}}
						style={{ left: contextMenu.x, top: contextMenu.y }}
					>
						<Button
							className="h-10 w-full justify-start rounded-[0.9rem]"
							onClick={openFolderCreateDialog}
							size="sm"
							variant="ghost"
						>
							{t("characters.folders.menuCreate")}
						</Button>
						<Button
							className="mt-1 h-10 w-full justify-start rounded-[0.9rem]"
							disabled={contextMenu.target !== "folder"}
							onClick={() => {
								if (contextMenu.target !== "folder") {
									return;
								}

								openFolderRenameDialog(contextMenu.folder);
							}}
							size="sm"
							variant="ghost"
						>
							{t("characters.folders.menuRename")}
						</Button>
						<Button
							className="mt-1 h-10 w-full justify-start rounded-[0.9rem]"
							disabled={contextMenu.target !== "folder"}
							onClick={() => {
								if (contextMenu.target !== "folder") {
									return;
								}

								openFolderDeleteDialog(contextMenu.folder);
							}}
							size="sm"
							variant="ghost"
						>
							{t("characters.folders.menuDelete")}
						</Button>
					</div>
				</div>
			) : null}

			<input
				accept=".chr,application/octet-stream"
				className="sr-only"
				onChange={(event) => {
					void handleImportSelection(event);
				}}
				ref={importInputRef}
				type="file"
			/>

			<WorkspacePanelShell className="flex min-h-0 flex-1">
				<Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
				<CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
					<SectionHeader
						actions={
							<div className="flex min-h-10 flex-wrap items-center justify-end gap-2.5 md:flex-nowrap">
								<ViewModeToggle onChange={setViewMode} value={viewMode} />

								{selectionMode ? (
									<>
										<Badge className="normal-case px-3.5 py-2" variant="subtle">
											{t("characters.selection.count", {
												count: selectedCharacterIds.length,
											})}
										</Badge>

										<IconButton
											disabled={filteredCharacters.length === 0}
											icon={<FontAwesomeIcon icon={faCheckDouble} />}
											label={t("characters.actions.selectAll")}
											onClick={() => {
												setSelectedCharacterIds(
													filteredCharacters.map(
														(character) => character.character_id,
													),
												);
											}}
											size="md"
											variant="secondary"
										/>

										<IconButton
											disabled={selectedCharacterIds.length === 0}
											icon={<FontAwesomeIcon icon={faTrashCan} />}
											label={t("characters.actions.deleteSelected")}
											onClick={() => {
												requestDelete(selectedCharacterIds);
											}}
											size="md"
											variant="danger"
										/>

										<IconButton
											icon={<FontAwesomeIcon icon={faXmark} />}
											label={t("characters.actions.cancelSelection")}
											onClick={exitSelectionMode}
											size="md"
											variant="secondary"
										/>
									</>
								) : (
									<>
										<IconButton
											icon={<FontAwesomeIcon icon={faSquareCheck} />}
											label={t("characters.actions.selectMode")}
											onClick={() => {
												setSelectedCharacterId(null);
												setSelectedCharacterIds([]);
												setSelectionMode(true);
											}}
											size="md"
											variant="secondary"
										/>

										<IconButton
											disabled={isCreatingSamples}
											icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
											label={
												isCreatingSamples
													? t("characters.actions.creatingSamples")
													: t("characters.actions.createSamples")
											}
											onClick={() => {
												setIsSampleDialogOpen(true);
											}}
											size="md"
											variant="secondary"
										/>

										<IconButton
											icon={<FontAwesomeIcon icon={faPlus} />}
											label={t("characters.actions.create")}
											onClick={openCreateDialog}
											size="md"
										/>

										<IconButton
											disabled={isImporting}
											icon={<FontAwesomeIcon icon={faUpload} />}
											label={
												isImporting
													? t("characters.actions.importing")
													: t("characters.actions.import")
											}
											onClick={() => {
												importInputRef.current?.click();
											}}
											size="md"
											variant="secondary"
										/>
									</>
								)}
							</div>
						}
						title={t("characters.title")}
					/>
				</CardHeader>

				<CardContent className="min-h-0 flex-1 overflow-hidden pt-6">
					<div className="flex h-full min-h-0 gap-5">
						<div className="relative hidden min-h-0 shrink-0 lg:block">
							<motion.aside
								animate={{
									opacity: isExplorerOpen ? 1 : 0.8,
									width: isExplorerOpen ? 248 : 0,
								}}
								className="h-full overflow-hidden"
								initial={false}
								transition={
									prefersReducedMotion
										? { duration: 0 }
										: { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
								}
							>
								<div
									className="h-full rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4"
									onContextMenu={isExplorerOpen ? openRootContextMenu : undefined}
								>
									<div className="flex h-full min-h-0 flex-col gap-4">
										<div className="flex items-center justify-between gap-3">
											<div>
												<p className="text-xs uppercase tracking-[0.18em] text-[var(--color-text-muted)]">
													{t("characters.folders.kicker")}
												</p>
												<h3 className="font-display text-2xl text-[var(--color-text-primary)]">
													{t("characters.folders.title")}
												</h3>
											</div>
											<IconButton
												icon={<FontAwesomeIcon icon={faPlus} />}
												label={t("characters.folders.menuCreate")}
												onClick={openFolderCreateDialog}
												size="sm"
												variant="secondary"
											/>
										</div>

										<div className="min-h-0 space-y-2 overflow-y-auto pr-1">
											<FolderTreeItem
												active={activeFolder === "all"}
												count={characters.length}
												icon={<FontAwesomeIcon icon={faFolderOpen} />}
												label={t("characters.folders.all")}
												onClick={() => {
													setActiveFolder("all");
												}}
											/>
											<FolderTreeItem
												active={activeFolder === "__unfiled__"}
												count={
													characters.filter(
														(character) =>
															normalizeCharacterFolder(character.folder).length === 0,
													).length
												}
												dragActive={folderDropTarget === "__unfiled__"}
												icon={<FontAwesomeIcon icon={faFolder} />}
												label={t("characters.filters.unfiled")}
												onClick={() => {
													setActiveFolder("__unfiled__");
												}}
												onDragLeave={() => {
													handleFolderDragLeave("__unfiled__");
												}}
												onDragOver={(event) => {
													handleFolderDragOver(event, "__unfiled__");
												}}
												onDrop={(event) => {
													handleFolderDrop(event, "__unfiled__");
												}}
											/>
											{availableFolders
												.filter((folder) => folder.length > 0)
												.map((folder) => (
													<FolderTreeItem
														active={activeFolder === folder}
														count={
															characters.filter(
																(character) =>
																	normalizeCharacterFolder(character.folder) === folder,
															).length
														}
														dragActive={folderDropTarget === folder}
														icon={
															<FontAwesomeIcon
																icon={
																	activeFolder === folder ? faFolderOpen : faFolder
																}
															/>
														}
														key={folder}
														label={folder}
														onClick={() => {
															setActiveFolder(folder);
														}}
														onContextMenu={(event) => {
															openFolderContextMenu(event, folder);
														}}
														onDragLeave={() => {
															handleFolderDragLeave(folder);
														}}
														onDragOver={(event) => {
															handleFolderDragOver(event, folder);
														}}
														onDrop={(event) => {
															handleFolderDrop(event, folder);
														}}
													/>
												))}
										</div>
									</div>
								</div>
							</motion.aside>

							<button
								aria-label={
									isExplorerOpen
										? t("characters.folders.toggleCollapse")
										: t("characters.folders.toggleExpand")
								}
								className="absolute left-full top-1/2 z-10 flex h-24 w-4 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel-strong)_92%,transparent)] text-[11px] text-[var(--color-text-muted)] shadow-[var(--shadow-floating)] transition hover:w-5 hover:text-[var(--color-text-primary)]"
								onClick={() => {
									setIsExplorerOpen((current) => !current);
								}}
								type="button"
							>
								<FontAwesomeIcon
									icon={isExplorerOpen ? faChevronLeft : faChevronRight}
								/>
							</button>
						</div>

						<div className="min-h-0 flex-1 overflow-y-auto">
							<div className="space-y-5">
								<div className="space-y-4 rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
									<div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto]">
										<label className="relative block">
											<span className="pointer-events-none absolute left-4 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
												<FontAwesomeIcon icon={faMagnifyingGlass} />
											</span>
											<Input
												className="pl-11"
												onChange={(event) => {
													setSearchQuery(event.target.value);
												}}
												placeholder={t("characters.filters.searchPlaceholder")}
												value={searchQuery}
											/>
										</label>

										<Badge
											className="h-11 self-start px-4 py-3 normal-case"
											variant="subtle"
										>
											{t("characters.filters.results", {
												count: filteredCharacters.length,
											})}
										</Badge>
									</div>
								</div>

								{characters.length === 0 && !isLoading ? (
									<div className="rounded-[1.6rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-6 py-12 text-center">
										<h3 className="font-display text-3xl text-[var(--color-text-primary)]">
											{t("characters.empty.title")}
										</h3>

										<div className="mt-7 flex flex-wrap justify-center gap-3">
											<Button
												onClick={() => {
													openCreateDialog();
												}}
												size="md"
											>
												{t("characters.actions.create")}
											</Button>
											<Button
												disabled={isImporting}
												onClick={() => {
													importInputRef.current?.click();
												}}
												size="md"
												variant="secondary"
											>
												{t("characters.actions.import")}
											</Button>
										</div>
									</div>
								) : filteredCharacters.length === 0 && !isLoading ? (
									<div className="rounded-[1.6rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-6 py-12 text-center">
										<h3 className="font-display text-3xl text-[var(--color-text-primary)]">
											{t("characters.filters.emptyTitle")}
										</h3>
										<p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
											{t("characters.filters.emptyDescription")}
										</p>
									</div>
								) : (
									<>
										{activeFolder !== "all" ? (
											<div className="flex items-center gap-3">
												<Badge className="normal-case px-3 py-1" variant="info">
													{t("characters.filters.currentFolder")}
												</Badge>
												<p className="text-sm text-[var(--color-text-secondary)]">
													{getCharacterFolderLabel(
														activeFolder === "__unfiled__" ? "" : activeFolder,
														t("characters.filters.unfiled"),
													)}
												</p>
											</div>
										) : null}

										<CharacterResults
											characters={filteredCharacters}
											coverUrls={coverUrls}
											isLoading={isLoading}
											onDelete={(characterId) => {
												requestDelete([characterId]);
											}}
											onDragEnd={handleCharacterDragEnd}
											onDragStart={handleCharacterDragStart}
											onEdit={openEditDialog}
											onOpenDetails={setSelectedCharacterId}
											onToggleSelect={toggleCharacterSelection}
											selectedCharacterIds={selectedCharacterSet}
											selectionMode={selectionMode}
											viewMode={viewMode}
										/>
									</>
								)}
							</div>
						</div>
					</div>
				</CardContent>
				</Card>
			</WorkspacePanelShell>
		</div>
	);
}
