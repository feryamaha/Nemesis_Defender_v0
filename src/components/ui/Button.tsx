"use client";
import clsx from "clsx";
import React from "react";
import { twMerge } from "tailwind-merge";
import Link from "next/link";
import { Icon } from "@/script/Icon";
import type { ButtonProps } from "@/types/dashboard-layout/button.types";
import { useButton } from "@/hooks/dashboard-layout/useButton.hook";

export function Button({
  variant,
  size,
  disabled,
  background,
  icon = "none",
  positionIcon = "left",
  href,
  target,
  rel,
  ...props
}: ButtonProps) {
  const { className, ...restProps } = props;

  const classes = twMerge(
    clsx(
      "font-inter leading-[150%] text-base font-semibold rounded-md flex  items-center duration-200 ease-linear transition-colors cursor-pointer w-full",

      variant === "primary" &&
      (disabled
        ? "bg-secondary-200 text-secondary-400 gap-[8px] cursor-not-allowed justify-center"
        : "bg-primary-600 text-white hover:bg-primary-700 gap-[8px] justify-center"),

      variant === "primary-icon" &&
      (disabled
        ? "bg-secondary-200 text-secondary-400 gap-[8px] cursor-not-allowed justify-between"
        : "bg-primary-600 text-white hover:bg-primary-700 gap-[8px] justify-between"),

      variant === "secondary" &&
      (disabled
        ? "font-medium border border-stroke-100 text-neutral-200 justify-between rounded-lg cursor-not-allowed justify-between"
        : "font-medium border border-stroke-100 text-secondary-950 justify-between rounded-lg bg-white hover:border-stroke-200 justify-between shadow-[0_1px_4px_0_rgba(0,0,0,0.08),0_1px_2px_0_rgba(25,25,25,0.08)]"),

      variant === "tertiary" &&
      (disabled
        ? "text-secondary-200 gap-[12px] cursor-not-allowed justify-center"
        : "text-primary-500 hover:text-primary-700 gap-[8px] shadow-none justify-center"),

      variant === "link" &&
      (disabled
        ? "text-secondary-200 font-normal gap-[12px] cursor-not-allowed"
        : "text-primary-500 font-medium leading-[140%] hover:underline underline-offset-4  gap-[8px] shadow-none"),
      size === "lg" && "p-[12px_24px]",
      size === "md" && "p-[8px_16px]",
      size === "sm" && "p-[7px_16px]",
      size === "default" && "p-0",
      background === "dangerPrimary" &&
      (disabled
        ? "bg-secondary-200 text-secondary-400 cursor-not-allowed"
        : "bg-white text-redSTD border border-redSTD hover:text-primary-700 hover:border-primary-700"),
      background === "dangerSecondary" &&
      (disabled
        ? "border border-secondary-100 bg-white text-secondary-200 gap-[8px] cursor-not-allowed"
        : "bg-danger-secondary-bg text-danger-secondary-text border border-danger-secondary-border hover:bg-danger-secondary-hover-bg hover:border-danger-secondary-hover-border hover:text-danger-secondary-hover-text gap-[8px]"),
      background === "dangerTertiary" &&
      (disabled
        ? "text-secondary-200 gap-[12px] cursor-not-allowed"
        : "text-danger-tertiary-text hover:text-danger-tertiary-hover-text underline-offset-4 gap-[8px]")
    )
  );

  const mergedClassName = twMerge(
    classes,
    className // Classes adicionais do usuário
  );

  const { positionIcon: resolvedPositionIcon, processedChildren, autoIconName, onMouseEnter, onMouseLeave } = useButton({
    variant,
    size,
    disabled,
    background,
    icon,
    positionIcon,
    children: props.children,
  });

  const autoIcon = autoIconName ? <Icon name={autoIconName} /> : null;

  const buttonContent = (
    <>
      {resolvedPositionIcon === "left" && autoIcon}
      {processedChildren}
      {resolvedPositionIcon === "right" && autoIcon}
    </>
  );

  const commonButtonProps = {
    className: `${mergedClassName} focus-visible:outline-2 focus-visible:outline-primary-500 focus-visible:outline-offset-2`,
    onMouseEnter,
    onMouseLeave,
    ...restProps,
  };

  if (href) {
    const isExternal =
      href.startsWith("http://") || href.startsWith("https://");
    const resolvedTarget = target ?? (isExternal ? "_blank" : undefined);
    const resolvedRel =
      rel ?? (resolvedTarget === "_blank" ? "noopener" : undefined);
    const linkProps =
      resolvedTarget || resolvedRel
        ? { target: resolvedTarget, rel: resolvedRel }
        : {};
    return (
      <Link href={href} className="flex" {...linkProps}>
        <button
          {...commonButtonProps}
          onClick={disabled ? (e) => e.preventDefault() : undefined}
        >
          {buttonContent}
        </button>
      </Link>
    );
  }

  return (
    <button {...commonButtonProps} disabled={disabled}>
      {buttonContent}
    </button>
  );
}