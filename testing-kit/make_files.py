"""Builds the testing kit's sample files: two PDFs, a DOCX and the voice memo.

Run from the repository root: python testing-kit/make_files.py
Needs reportlab (pip install reportlab); the voice memo needs Windows (built-in speech) and ffmpeg.
"""

import pathlib
import subprocess
import sys
import zipfile
from xml.sax.saxutils import escape

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet
from reportlab.platypus import Paragraph, SimpleDocTemplate, Spacer

OUT = pathlib.Path(__file__).parent / "files"

BILLING = ("Larkspur billing and payments FAQ", "Updated 1 September 2026", [
    ("Which payment methods do you accept?",
     "We accept Visa, Mastercard and American Express on every plan. Bank transfer is available for annual plans only."),
    ("Is annual billing cheaper?",
     "Yes. On an annual plan you pay for 10 months and get 12."),
    ("When are invoices sent?",
     "Invoices are emailed to the account owner on the 3rd of each month. Every invoice number starts with INV-, "
     "for example INV-2291."),
    ("What happens if a card payment fails?",
     "We retry the card 3 times over 7 days and email the account owner after each attempt. If the payment still "
     "fails, the account becomes read-only on day 10 until the invoice is paid."),
    ("Do you charge VAT?",
     "Customers in the European Union are charged VAT unless they enter a valid VAT ID in Settings, Billing."),
    ("Which currencies can I pay in?",
     "US dollars, euros and British pounds."),
    ("How do refunds work?",
     "Refunds follow the current refund policy: a full refund within 30 days of the invoice date. Refunds above "
     "$100 are reviewed by a person before they are paid."),
])

SECURITY = ("Larkspur security and privacy overview", "Updated 15 July 2026", [
    ("Where is customer data stored?",
     "All customer data is stored in Frankfurt, Germany, on disks encrypted with AES-256. Data in transit uses TLS 1.3."),
    ("Do you have a SOC 2 report?",
     "Yes, a SOC 2 Type II report. Customers on the Scale plan can request it under an NDA."),
    ("Do you support single sign-on?",
     "SAML single sign-on is available on the Scale plan only."),
    ("Is two-factor authentication required?",
     "Two-factor authentication has been required for every workspace admin since 1 June 2026."),
    ("What happens to my data when I close my account?",
     "We delete account data within 30 days of closure. Backups are purged after 35 days."),
    ("How do I report a security issue?",
     "Email security@larkspur.example. Valid reports can earn a reward of up to $2,000."),
])

ONBOARDING = ("Larkspur onboarding guide", [
    "Welcome to Larkspur. This guide gets a new team live in under an hour.",
    "Free trial: every new workspace gets a 14-day free trial of the Team plan. No card is needed to start.",
    "Step 1, create your workspace: sign up at larkspur.example and choose a workspace name.",
    "Step 2, invite teammates: open Settings, Team. Starter includes 1 seat, Team 10 seats and Scale 50 seats.",
    "Step 3, connect a channel: email and the website chat widget work on every plan. WhatsApp is available on "
    "Team and Scale.",
    "Step 4, import contacts: upload a CSV file of up to 50,000 rows at a time. Larger lists can be split into "
    "several files.",
    "Step 5, set business hours: replies sent outside business hours are queued for the next morning.",
    "Attachments: customers can send files of up to 25 MB.",
    "Onboarding call: Team and Scale customers can book a free 30-minute onboarding call at "
    "larkspur.example/onboarding.",
])

MEMO = (
    "Hi, this is Priya, head of support at Larkspur. A few rules for the support assistant. "
    "Be warm and brief: no more than three short paragraphs. Greet the customer by first name when you know it. "
    "Never promise a refund above one hundred dollars. Say that a person will review it within one business day. "
    "If a customer sounds angry, or mentions cancelling, apologise once and offer to open a ticket so a person "
    "can follow up. Never discuss another customer's account, not even their name or email. "
    "If the answer isn't in our documents, say you don't know and offer a ticket. Never guess. "
    "And finally, end every answer with: Team Larkspur. Thanks!"
)


def pdf(name: str, title: str, dated: str, faq: list[tuple[str, str]]) -> None:
    styles = getSampleStyleSheet()
    story = [Paragraph(escape(title), styles["Title"]), Paragraph(escape(dated), styles["Italic"]), Spacer(1, 12)]
    for question, answer in faq:
        story += [Paragraph(escape(question), styles["Heading3"]), Paragraph(escape(answer), styles["BodyText"])]
    SimpleDocTemplate(str(OUT / name), pagesize=A4, title=title, author="Larkspur (sample)").build(story)


def docx(name: str, title: str, paragraphs: list[str]) -> None:
    def para(text: str, style: str = "") -> str:
        props = f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>' if style else ""
        return f'<w:p>{props}<w:r><w:t xml:space="preserve">{escape(text)}</w:t></w:r></w:p>'

    body = para(title, "Title") + "".join(para(p) for p in paragraphs)
    ns = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
    with zipfile.ZipFile(OUT / name, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml",
                   '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
                   '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
                   '<Default Extension="xml" ContentType="application/xml"/>'
                   '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
                   '</Types>')
        z.writestr("_rels/.rels",
                   '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
                   '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
                   '</Relationships>')
        z.writestr("word/document.xml",
                   f'<?xml version="1.0" encoding="UTF-8"?><w:document {ns}><w:body>{body}</w:body></w:document>')


def memo(name: str, text: str) -> None:
    wav = OUT / "memo.wav"
    script = (
        "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; "
        f"$s.Rate = -1; $s.SetOutputToWaveFile('{wav}'); $s.Speak($env:MEMO); $s.Dispose()"
    )
    subprocess.run(["powershell", "-NoProfile", "-Command", script], check=True, env={**__import__("os").environ, "MEMO": text})
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(wav), "-ac", "1", "-b:a", "64k", str(OUT / name)], check=True)
    wav.unlink()


if __name__ == "__main__":
    OUT.mkdir(exist_ok=True)
    pdf("larkspur-billing-faq.pdf", *BILLING)
    pdf("larkspur-security-privacy.pdf", *SECURITY)
    docx("larkspur-onboarding-guide.docx", *ONBOARDING)
    if sys.platform == "win32":
        memo("support-rules-voice-memo.mp3", MEMO)
    print("files written to", OUT)
