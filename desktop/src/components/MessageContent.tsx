import type { JSX, ReactNode } from 'react';

interface MessageContentProps {
  content: string;
  className?: string;
}

interface ProseMirrorNode {
  type: string;
  text?: string;
  attrs?: Record<string, unknown>;
  content?: ProseMirrorNode[];
  marks?: Array<{ type: string; attrs?: Record<string, unknown> }>;
}

function isMentionNode(node: ProseMirrorNode): boolean {
  return node.type === 'mention' && typeof node.attrs?.id === 'string';
}

function renderMarks(text: string, marks: ProseMirrorNode['marks']): ReactNode {
  let result: ReactNode = text;
  if (!marks) {
    return result;
  }
  for (const mark of marks) {
    switch (mark.type) {
      case 'bold':
      case 'strong':
        result = <strong>{result}</strong>;
        break;
      case 'italic':
      case 'em':
        result = <em>{result}</em>;
        break;
      case 'strike':
        result = <s>{result}</s>;
        break;
      case 'code':
        result = <code className="rounded bg-surface-elevated px-1 py-0.5 text-xs">{result}</code>;
        break;
      case 'link':
        if (typeof mark.attrs?.href === 'string') {
          result = (
            <a
              href={mark.attrs.href}
              target="_blank"
              rel="noreferrer"
              className="text-accent underline hover:text-accent-hover"
            >
              {result}
            </a>
          );
        }
        break;
      default:
        break;
    }
  }
  return result;
}

function renderNode(node: ProseMirrorNode, index: number): ReactNode {
  if (node.type === 'text' && node.text !== undefined) {
    return <span key={index}>{renderMarks(node.text, node.marks)}</span>;
  }

  if (isMentionNode(node)) {
    const label =
      typeof node.attrs?.label === 'string' ? node.attrs.label : String(node.attrs?.id ?? '');
    return (
      <span
        key={index}
        className="rounded bg-accent-bg px-1 py-0.5 font-medium text-accent"
      >
        @{label}
      </span>
    );
  }

  if (node.type === 'paragraph') {
    return (
      <p key={index} className="min-h-[1em]">
        {node.content?.map((child, i) => renderNode(child, i)) ?? <br />}
      </p>
    );
  }

  if (node.type === 'doc') {
    return (
      <div key={index}>
        {node.content?.map((child, i) => renderNode(child, i))}
      </div>
    );
  }

  if (node.type === 'bulletList') {
    return (
      <ul key={index} className="list-inside list-disc">
        {node.content?.map((child, i) => renderNode(child, i))}
      </ul>
    );
  }

  if (node.type === 'orderedList') {
    return (
      <ol key={index} className="list-inside list-decimal">
        {node.content?.map((child, i) => renderNode(child, i))}
      </ol>
    );
  }

  if (node.type === 'listItem') {
    return <li key={index}>{node.content?.map((child, i) => renderNode(child, i))}</li>;
  }

  if (node.type === 'blockquote') {
    return (
      <blockquote key={index} className="border-l-4 border-border pl-3 italic text-text-muted">
        {node.content?.map((child, i) => renderNode(child, i))}
      </blockquote>
    );
  }

  if (node.type === 'codeBlock') {
    return (
      <pre key={index} className="overflow-x-auto rounded-md bg-surface-elevated p-2 text-xs">
        <code>{node.content?.map((child) => child.text ?? '').join('')}</code>
      </pre>
    );
  }

  if (node.type === 'hardBreak') {
    return <br key={index} />;
  }

  // Fallback for unknown nodes: render their children inline.
  return (
    <span key={index}>
      {node.content?.map((child, i) => renderNode(child, i))}
    </span>
  );
}

function tryParseContent(content: string): ProseMirrorNode | null {
  try {
    const parsed = JSON.parse(content) as unknown;
    if (parsed && typeof parsed === 'object' && 'type' in parsed) {
      return parsed as ProseMirrorNode;
    }
  } catch {
    // Fall through to plain text rendering.
  }
  return null;
}

export function MessageContent({ content, className = '' }: MessageContentProps): JSX.Element {
  const doc = tryParseContent(content);
  if (!doc) {
    return <div className={`whitespace-pre-wrap ${className}`}>{content}</div>;
  }
  return <div className={`whitespace-pre-wrap ${className}`}>{renderNode(doc, 0)}</div>;
}
